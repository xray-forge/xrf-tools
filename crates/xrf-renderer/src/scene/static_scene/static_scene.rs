use std::collections::VecDeque;
use std::sync::Arc;

use glam::{Mat4, UVec4, Vec3, Vec4};
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer_core::{ProxyHandle, ProxyStore, Span};
use xrf_visual::{SectorGeometry, SectorImpostors, SectorInstanceGroup, SectorPackage, SectorSurface};

use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::contract::render_static_report::RenderStaticReport;
use crate::frame::frame_encoder::FrameEncoder;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::static_geometry::StaticGeometry;
use crate::scene::static_scene::static_geometry_base::StaticGeometryBase;
use crate::scene::static_scene::static_impostor::StaticImpostor;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_model::StaticModel;
use crate::scene::static_scene::static_model_place::StaticModelPlace;
use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::static_objects::StaticObjects;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_row::StaticRow;
use crate::scene::static_scene::static_scene_writer::StaticSceneWriter;
use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::static_scene::static_sight::StaticSight;
use crate::scene::static_scene::static_slot::StaticSlot;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::static_sorted_place::StaticSortedPlace;
use crate::scene::static_scene::static_sorted_run::StaticSortedRun;
use crate::scene::static_scene::static_spans::StaticSpans;
use crate::scene::static_scene::static_surface_key::StaticSurfaceKey;
use crate::scene::static_scene::static_surface_use::StaticSurfaceUse;
use crate::scene::static_scene::static_surfaces::StaticSurfaces;
use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::static_trace_group::StaticTraceGroup;
use crate::scene::static_scene::static_vertex_words::StaticVertexWords;
use crate::scene::texture::texture_cache::TextureCache;

/// Clusters a spawned model's row holds at most: the row cull walks a row's clusters on one thread.
const MODEL_ROW_CLUSTERS: usize = 32;

/// A level's static geometry on the GPU, as sectors, spawned models and the objects standing as them are added and
/// removed: the geometry's buffers each item takes ranges of, the surfaces every item shares, what a frame's culls
/// write into, and the items themselves, found by handle.
pub struct StaticScene {
  pub geometry: StaticGeometry,
  pub surfaces: StaticSurfaces,
  pub cull: StaticCullScratch,
  sectors: ProxyStore<StaticSectorProxy>,
  models: ProxyStore<StaticModelProxy>,
  objects: StaticObjects,
  /// The bounding sphere of every place that sways and its reach, which a light's shadow looks for in its range.
  swaying: Vec<(Vec4, f32)>,
  /// The farthest any swaying vertex reaches from its tree's foot, weighted by its rigidity and scaled by its place.
  sway_reach: f32,
  /// Items added and removed, which a view drawn with another count is drawn again for.
  contents: usize,
  /// The sphere in the world of each item added or removed lately, the first the change after `contents` was
  /// `changes_from`; none (a negative radius) for an item standing nowhere, a model alone.
  changes: VecDeque<Vec4>,
  changes_from: usize,
}

impl StaticScene {
  /// Changes the log keeps, past which a cache drawn before them all is drawn again whole.
  const CHANGES_KEPT: usize = 1024;

  /// Where an item standing nowhere of its own changes the scene: no sphere reaches it.
  const NOWHERE: Vec4 = Vec4::new(0.0, 0.0, 0.0, -1.0);

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    Self {
      geometry: StaticGeometry::new(device, queue),
      surfaces: StaticSurfaces::new(device, queue),
      cull: StaticCullScratch::new(device),
      sectors: ProxyStore::new(),
      models: ProxyStore::new(),
      objects: StaticObjects::default(),
      swaying: Vec::new(),
      sway_reach: 0.0,
      contents: 0,
      changes: VecDeque::new(),
      changes_from: 0,
    }
  }

  /// How full the scene's pools are: what a report of its static draws starts from.
  pub fn get_pools(&self) -> RenderStaticReport {
    RenderStaticReport {
      surface_list: RenderPoolUse {
        used: 0,
        capacity: self.cull.get_list_capacity(),
      },
      ..self.geometry.get_pools()
    }
  }

  /// The sector and shader table entry of a picked impostor.
  pub fn resolve_impostor_pick(&self, impostor: u32) -> Option<(u32, u16)> {
    self
      .sectors
      .as_slice()
      .iter()
      .find(|sector| StaticSpans::contains(&sector.spans.impostors, impostor))
      .and_then(|sector| {
        let local: u32 = impostor - StaticSpans::get_offset(&sector.spans.impostors);

        Some((sector.sector, *sector.impostor_shaders.get(local as usize)?))
      })
  }

  /// What a pick's cluster and place were part of: its slot's part, and the place's instance where it is a mesh's.
  pub fn resolve_pick(&self, cluster: u32, place: u32) -> Option<(StaticSlotInfo, Option<u32>)> {
    let (spans, slot_infos, cluster_slots): (&StaticSpans, &[StaticSlotInfo], &[u32]) = self
      .sectors
      .as_slice()
      .iter()
      .map(|sector| {
        (
          &sector.spans,
          sector.slot_infos.as_slice(),
          sector.cluster_slots.as_slice(),
        )
      })
      .chain(self.models.as_slice().iter().map(|model| {
        (
          &model.spans,
          model.slot_infos.as_slice(),
          model.cluster_slots.as_slice(),
        )
      }))
      .find(|(spans, _, _)| StaticSpans::contains(&spans.clusters, cluster))?;
    let local: u32 = cluster - StaticSpans::get_offset(&spans.clusters);
    let info: StaticSlotInfo = *slot_infos.get(*cluster_slots.get(local as usize)? as usize)?;

    Some((info, info.mesh.map(|_| place.saturating_sub(info.first_place))))
  }

  /// The spawned object a picked place is, where it is one.
  pub fn resolve_spawn_pick(&self, place: u32) -> Option<u32> {
    self.objects.find_at(place).map(|object| object.object)
  }

  /// Whether a spawned object stands in the scene.
  pub fn has_object(&self, object: u32) -> bool {
    self.objects.contains(object)
  }

  /// A spawned object's bounding sphere in renderer space, while it stands in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.objects.find(object).map(|object| object.sphere)
  }

  /// A spawned object's box as it stands: its model's box in its own space, and where it stands it.
  pub fn get_object_box(&self, object: u32) -> Option<(Mat4, [Vec3; 2])> {
    let object: &StaticObjectProxy = self.objects.find(object)?;

    Some((object.transform, self.models.get(object.model)?.bounds))
  }

  /// What a selection marks, once what it names is in the scene: a spawned object's place; a mesh's place; or the
  /// runs of clusters drawing a sector's baked geometry of one shader table entry.
  pub fn resolve_selection(&self, target: &RenderSelectionTarget) -> Option<StaticSelection> {
    match *target {
      RenderSelectionTarget::Spawn { object } => Some(StaticSelection {
        place: Some(self.objects.find(object)?.place.get_offset()),
        runs: vec![StaticSelection::ANY_CLUSTER],
      }),
      RenderSelectionTarget::Surface {
        sector,
        mesh: Some(mesh),
        place: Some(place),
        ..
      } => {
        let info: &StaticSlotInfo = self
          .find_sector(sector)?
          .slot_infos
          .iter()
          .find(|info| info.mesh == Some(mesh))?;

        Some(StaticSelection {
          place: Some(info.first_place + place),
          runs: vec![StaticSelection::ANY_CLUSTER],
        })
      }
      RenderSelectionTarget::Surface { sector, shader_id, .. } => {
        let proxy: &StaticSectorProxy = self.find_sector(sector)?;
        let first: u32 = StaticSpans::get_offset(&proxy.spans.clusters);
        let mut runs: Vec<[u32; 2]> = Vec::new();

        for (local, slot) in proxy.cluster_slots.iter().enumerate() {
          let Some(info) = proxy.slot_infos.get(*slot as usize) else {
            continue;
          };

          if u32::from(info.shader_id) != shader_id || info.mesh.is_some() {
            continue;
          }

          let cluster: u32 = first + local as u32;

          match runs.last_mut() {
            Some(run) if run[1] == cluster => run[1] = cluster + 1,
            _ => runs.push([cluster, cluster + 1]),
          }
        }

        (!runs.is_empty()).then_some(StaticSelection { place: None, runs })
      }
    }
  }

  /// Stands a skinned object in a pose: each bone's matrix as three rows, from its bind to where it stands, this frame
  /// and the last, as many as its model has bones each; written with the scene's other changes.
  pub fn write_pose(&mut self, handle: ProxyHandle<StaticObjectProxy>, current: &[[Vec4; 3]], previous: &[[Vec4; 3]]) {
    let Some((span, bones)) = self.objects.get(handle).and_then(|object| object.bones.as_ref()) else {
      return;
    };
    let count: usize = *bones as usize;
    let mut rows: Vec<[Vec4; 3]> = vec![[Vec4::X, Vec4::Y, Vec4::Z]; count * 2];

    for (to, from) in rows[..count].iter_mut().zip(current) {
      *to = *from;
    }

    for (to, from) in rows[count..].iter_mut().zip(previous) {
      *to = *from;
    }

    self.geometry.bones.write(span, 0, bytemuck::cast_slice(&rows));
  }

  /// Puts one packed sector into the scene at the handle its poster allocated: its baked sections as single draws, its
  /// tree groups as rows.
  ///
  /// # Errors
  ///
  /// Returns an error when a buffer would outgrow what the device allows; the scene is left as it was.
  #[allow(clippy::too_many_arguments)]
  pub fn add_sector(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
    descriptors: &[XraySurfaceDescriptor],
    (handle, package): (ProxyHandle<StaticSectorProxy>, &SectorPackage),
  ) -> XrfResult {
    let mut writer: StaticSceneWriter = StaticSceneWriter::default();
    let mut swaying: Vec<(Vec4, f32)> = Vec::new();
    // What its baked sections hold, in the world.
    let mut sections: Vec<Vec4> = Vec::new();
    let description = &package.description;
    let buffer: &[u8] = &package.buffer;

    if !description.sections.is_empty() && description.geometry.index_count > 0 {
      let base: StaticGeometryBase =
        Self::put_geometry(&mut writer, StaticLayout::Baked, &description.geometry, buffer);

      for section in &description.sections {
        let Some(StaticSurfaceUse { surface, class, passes }) = self.surfaces.resolve(
          &mut writer,
          Self::level_surface(&section.surface, descriptors),
          textures,
          source,
        ) else {
          continue;
        };
        let batch: StaticBatch = StaticBatch {
          layout: StaticLayout::Baked,
          class,
        };
        let cut: Vec<(u32, u32, Vec4)> =
          Self::cut_clusters(&description.geometry, buffer, section.draw.start, section.draw.count);
        let sphere: Vec4 = Self::bound_spheres(&cut.iter().map(|(_, _, sphere)| *sphere).collect::<Vec<Vec4>>());

        sections.push(sphere);
        let (slot, clusters): (u32, u32) = writer.put_slot(
          &base,
          cut,
          (StaticSlot::SINGLE, batch, surface, passes),
          StaticSlotInfo {
            sector: description.sector,
            shader_id: section.surface.shader_id,
            mesh: None,
            first_place: 0,
          },
        );

        for list in batch.list_draw_lists(passes) {
          writer.capacities[list] += clusters;
        }

        if class == StaticClass::Sorted {
          writer.sorted_runs.push(StaticSortedRun {
            sphere,
            place: None,
            layout: StaticLayout::Baked,
            clusters: (writer.slots[slot as usize].first_cluster, clusters),
          });
        }

        let sight: StaticSight = StaticSight::of(class, passes);

        if !sight.is_empty() {
          writer.sights.push((sphere, sight));
        }

        if class.is_deferred() {
          writer.trace_geometries.push(StaticTraceGeometry {
            layout: StaticLayout::Baked,
            first_index: base.index_start + section.draw.start,
            index_count: section.draw.count,
            vertex_start: base.vertex_start,
            vertex_count: description.geometry.vertex_count,
            surface,
            is_cut_out: class == StaticClass::CutOut,
          });
        }
      }
    }

    if let Some(impostors) = &description.impostors {
      self.put_impostors(&mut writer, impostors, buffer, textures, source);
    }

    for (mesh, group) in description.instances.iter().enumerate() {
      self.put_instances(
        (&mut writer, &mut swaying),
        (description.sector, mesh as u32),
        group,
        buffer,
        descriptors,
        textures,
        source,
      );
    }

    self.surfaces.put(device, queue, encoder, &writer);

    let spans: StaticSpans = self.geometry.allocate(device, queue, &writer)?;

    writer.relocate(&spans);
    self.geometry.write(&spans, &writer);

    let rows: Vec<ProxyHandle<StaticRow>> = writer.rows.iter().map(|row| self.geometry.add_row(*row)).collect();

    let sphere: Vec4 = Self::bound_spheres(
      &sections
        .into_iter()
        .chain(writer.rows.iter().map(|row| row.sphere))
        .chain(
          writer
            .impostors
            .iter()
            .map(|impostor| impostor.sphere)
            .filter(|it| it.w >= 0.0),
        )
        .filter(|it| it.w >= 0.0)
        .collect::<Vec<Vec4>>(),
    );

    self.cull.add_capacities(&writer.capacities);
    self.sway_reach = swaying.iter().map(|(_, reach)| *reach).fold(self.sway_reach, f32::max);
    self.swaying.extend_from_slice(&swaying);
    self.note_change(sphere);

    self.sectors.insert(
      handle,
      StaticSectorProxy {
        sphere,
        sector: description.sector,
        bytes: buffer.len() as u64,
        spans,
        rows,
        capacities: writer.capacities,
        slot_infos: writer.slot_infos,
        cluster_slots: writer.cluster_slots,
        impostor_shaders: writer.impostor_shaders,
        trace_geometries: writer.trace_geometries,
        trace_groups: writer.trace_groups,
        sorted_runs: writer.sorted_runs,
        sights: writer.sights,
        swaying,
      },
    );

    Ok(())
  }

  /// Whether the device traces rays, so the vertices and indices are acceleration structures' input.
  pub fn is_traced(&self) -> bool {
    self.geometry.is_traced()
  }

  /// Every sector in the scene with its deferred baked sections and instanced meshes, which the level's acceleration
  /// structures take.
  pub fn list_trace_sectors(&self) -> impl Iterator<Item = (u32, &[StaticTraceGeometry], &[StaticTraceGroup])> {
    self.sectors.iter().map(|(_, sector)| {
      (
        sector.sector,
        sector.trace_geometries.as_slice(),
        sector.trace_groups.as_slice(),
      )
    })
  }

  /// Every spawned model the scene's objects stand as that the level's acceleration structures take, with its traced
  /// parts.
  pub fn list_trace_models(&self) -> impl Iterator<Item = (ProxyHandle<StaticModelProxy>, &[StaticTraceGeometry])> {
    self
      .models
      .iter()
      .filter(|(_, model)| !model.trace_geometries.is_empty())
      .map(|(handle, model)| (handle, model.trace_geometries.as_slice()))
  }

  /// Every spawned object standing as a traced model: its model, its place and its transform.
  pub fn list_trace_objects(&self) -> impl Iterator<Item = (ProxyHandle<StaticModelProxy>, u32, &Mat4)> {
    self
      .objects
      .iter()
      .filter(|(_, object)| {
        self
          .models
          .get(object.model)
          .is_some_and(|model| !model.trace_geometries.is_empty())
      })
      .map(|(_, object)| (object.model, object.place.get_offset(), &object.transform))
  }

  /// Takes a sector out of the scene, its ranges cleared and freed; false for one already gone.
  pub fn remove_sector(&mut self, handle: ProxyHandle<StaticSectorProxy>) -> bool {
    let Some(sector) = self.sectors.remove(handle) else {
      return false;
    };

    for row in sector.rows {
      self.geometry.remove_row(row);
    }

    self.cull.remove_capacities(&sector.capacities);
    self.geometry.release(sector.spans);
    self.swaying = self
      .sectors
      .as_slice()
      .iter()
      .flat_map(|sector| sector.swaying.iter().copied())
      .collect();
    self.sway_reach = self.swaying.iter().map(|(_, reach)| *reach).fold(0.0, f32::max);
    self.note_change(sector.sphere);

    true
  }

  /// Puts a spawned model into the scene at the handle its poster allocated: its geometry once and a slot a part, which
  /// each object standing as it adds a row of.
  ///
  /// # Errors
  ///
  /// Returns an error when a buffer would outgrow what the device allows; the scene is left as it was.
  pub fn add_model(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    (textures, source): (&mut TextureCache, &Arc<dyn RenderAssetSource>),
    (handle, model): (ProxyHandle<StaticModelProxy>, &StaticModel),
  ) -> XrfResult {
    let mut writer: StaticSceneWriter = StaticSceneWriter::default();
    let base: StaticGeometryBase = writer.put_words(StaticLayout::Model, &model.words, &model.indices);
    let vertex_count: u32 = (model.words.len() / StaticLayout::STRIDE as usize) as u32;
    let mut slots: Vec<(u32, u32, StaticBatch, u32)> = Vec::new();
    // The composited parts' clusters, which each object lists to be drawn sorted.
    let mut sorted: Vec<(u32, u32)> = Vec::new();
    let mut sight: StaticSight = StaticSight::default();

    for part in &model.parts {
      let key: StaticSurfaceKey = StaticSurfaceKey::Model {
        shader: part.surface.shader_name.clone(),
        texture: part.surface.texture_name.clone(),
      };
      let Some(StaticSurfaceUse { surface, class, passes }) = self.surfaces.resolve(
        &mut writer,
        (key, &part.surface, part.descriptor.as_ref()),
        textures,
        source,
      ) else {
        continue;
      };
      let batch: StaticBatch = StaticBatch {
        layout: StaticLayout::Model,
        class,
      };

      if class.is_deferred() && model.skin.is_none() {
        writer.trace_geometries.push(StaticTraceGeometry {
          layout: StaticLayout::Model,
          first_index: base.index_start + part.indices.0,
          index_count: part.indices.1,
          vertex_start: base.vertex_start,
          vertex_count,
          surface,
          is_cut_out: class == StaticClass::CutOut,
        });
      }

      // A row's clusters are culled one after another by one thread, so a large part is cut into several slots.
      for clusters in part.clusters.chunks(MODEL_ROW_CLUSTERS) {
        let first_cluster: u32 = writer.clusters.len() as u32;
        let (slot, count): (u32, u32) = writer.put_slot(
          &base,
          clusters.to_vec(),
          (StaticSlot::LISTED, batch, surface, passes),
          StaticSlotInfo {
            sector: StaticSlotInfo::NO_SECTOR,
            shader_id: part.surface.shader_id,
            mesh: None,
            first_place: 0,
          },
        );

        if class == StaticClass::Composited {
          sorted.push((first_cluster, count));
        }

        sight |= StaticSight::of(class, passes);

        slots.push((slot, count, batch, passes));
      }
    }

    if let Some(skin) = &model.skin {
      writer.skins.extend_from_slice(&skin.links);
    }

    self.surfaces.put(device, queue, encoder, &writer);

    let spans: StaticSpans = self.geometry.allocate(device, queue, &writer)?;
    let first_slot: u32 = StaticSpans::get_offset(&spans.slots);
    let first_cluster: u32 = StaticSpans::get_offset(&spans.clusters);

    writer.relocate(&spans);
    self.geometry.write(&spans, &writer);
    self.note_change(Self::NOWHERE);

    self.models.insert(
      handle,
      StaticModelProxy {
        slots: slots
          .into_iter()
          .map(|(slot, count, batch, passes)| (first_slot + slot, count, batch, passes))
          .collect(),
        sorted: sorted
          .into_iter()
          .map(|(first, count)| (first_cluster + first, count))
          .collect(),
        sight,
        slot_infos: writer.slot_infos,
        cluster_slots: writer.cluster_slots,
        sphere: model.sphere,
        bounds: model.bounds,
        vertex_start: StaticSpans::get_offset(&spans.words[StaticLayout::Model.get_index()]) + base.vertex_start,
        skin: model
          .skin
          .as_ref()
          .map(|skin| (StaticSpans::get_offset(&spans.skins), skin.bones)),
        trace_geometries: writer.trace_geometries,
        spans,
      },
    );

    Ok(())
  }

  /// Takes a model out of the scene with every object standing as it, its ranges cleared and freed; false for one
  /// already gone.
  pub fn remove_model(&mut self, handle: ProxyHandle<StaticModelProxy>) -> bool {
    if !self.models.contains(handle) {
      return false;
    }

    for object in self.objects.list_standing(handle) {
      self.remove_object(object);
    }

    if let Some(model) = self.models.remove(handle) {
      self.geometry.release(model.spans);
    }

    self.note_change(Self::NOWHERE);

    true
  }

  /// Stands a spawned object as a model in the scene at the handle its poster allocated: a place, its bone matrices in
  /// the bind pose where the model is skinned, and a row a slot of the model.
  ///
  /// # Errors
  ///
  /// Returns an error for a model no longer in the scene, or when a buffer would outgrow what the device allows; the
  /// scene is left as it was.
  pub fn add_object(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    handle: ProxyHandle<StaticObjectProxy>,
    (model, place): (ProxyHandle<StaticModelProxy>, &StaticModelPlace),
  ) -> XrfResult {
    let Some(proxy) = self.models.get(model) else {
      return Err(XrfError::new_invalid_error(format!(
        "Spawned object {} stands as a model no longer in the scene",
        place.object
      )));
    };
    let slots: Vec<(u32, u32, StaticBatch, u32)> = proxy.slots.clone();
    let (sphere, vertex_start, skin): (Vec4, u32, Option<(u32, u32)>) = (proxy.sphere, proxy.vertex_start, proxy.skin);
    let matrix: Mat4 = place.transform;
    let scale: f32 = Self::to_scale(&matrix);
    let placed: Vec4 = matrix.transform_point3(sphere.truncate()).extend(sphere.w * scale);
    let span: Span = self.geometry.places.allocate(device, queue, 1)?;
    let bones: Option<(Span, u32)> = match skin {
      Some((_, bones)) => {
        let bones: u32 = bones.max(1);

        match self.geometry.bones.allocate(device, queue, bones * 2 * 3) {
          // This frame's matrices and the last's, both the bind pose until a pose is written.
          Ok(rows) => {
            let bind: Vec<[Vec4; 3]> = vec![[Vec4::X, Vec4::Y, Vec4::Z]; bones as usize * 2];

            self.geometry.bones.write(&rows, 0, bytemuck::cast_slice(&bind));

            Some((rows, bones))
          }
          Err(error) => {
            self.geometry.places.free(span);

            return Err(error);
          }
        }
      }
      None => None,
    };
    let index: u32 = span.get_offset();

    self.geometry.places.write(
      &span,
      0,
      bytemuck::bytes_of(&StaticPlace {
        transform: matrix,
        info: Vec4::new(1.0, 0.0, -1.0, scale),
        cube: place.lighting.map_or(UVec4::ZERO, Self::pack_cube),
        skin: match (&bones, skin) {
          (Some((rows, bones)), Some((links, _))) => UVec4::new(rows.get_offset(), links, vertex_start, *bones),
          _ => UVec4::ZERO,
        },
      }),
    );

    let mut capacities: [u32; StaticBatch::LIST_COUNT] = [0; StaticBatch::LIST_COUNT];
    let rows: Vec<ProxyHandle<StaticRow>> = slots
      .iter()
      .map(|(slot, clusters, batch, passes)| {
        for list in batch.list_draw_lists(*passes) {
          capacities[list] += clusters;
        }

        self.geometry.add_row(StaticRow {
          sphere: placed,
          place: index,
          slot: *slot,
          lod: StaticRow::NO_LOD,
          band: StaticRow::pack_band(0, 1, 1),
        })
      })
      .collect();

    self.cull.add_capacities(&capacities);
    self.note_change(placed);

    let replaced: Option<ProxyHandle<StaticObjectProxy>> = self.objects.insert(
      handle,
      StaticObjectProxy {
        object: place.object,
        model,
        place: span,
        bones,
        rows,
        capacities,
        transform: matrix,
        sphere: placed,
      },
    );

    // An object stood again replaces where it stood.
    if let Some(replaced) = replaced {
      self.remove_object(replaced);
    }

    Ok(())
  }

  /// Takes a spawned object out of the scene: its place, bone matrices and rows; false for one already gone.
  pub fn remove_object(&mut self, handle: ProxyHandle<StaticObjectProxy>) -> bool {
    let Some(object) = self.objects.remove(handle) else {
      return false;
    };

    for row in object.rows {
      self.geometry.remove_row(row);
    }

    self.cull.remove_capacities(&object.capacities);
    self.geometry.places.free(object.place);

    if let Some((bones, _)) = object.bones {
      self.geometry.bones.free(bones);
    }

    self.note_change(object.sphere);

    true
  }

  /// Items added and removed so far: what a cache notes it was drawn with, and asks `is_changed_since` about.
  pub fn get_contents(&self) -> usize {
    self.contents
  }

  /// Whether an item added or removed since the scene held `since` changes stands where `touches` says a sphere in the
  /// world reaches what a cache holds; anything did where the log no longer reaches back that far.
  pub fn is_changed_since(&self, since: usize, touches: impl Fn(Vec4) -> bool) -> bool {
    if since >= self.contents {
      return false;
    }

    if since < self.changes_from {
      return true;
    }

    self
      .changes
      .iter()
      .skip(since - self.changes_from)
      .any(|sphere| sphere.w >= 0.0 && touches(*sphere))
  }

  /// Counts a change of what the scene holds, at the sphere of the item added or removed.
  fn note_change(&mut self, sphere: Vec4) {
    self.contents += 1;
    self.changes.push_back(sphere);

    if self.changes.len() > Self::CHANGES_KEPT {
      self.changes.pop_front();
      self.changes_from += 1;
    }
  }

  /// Uploads what was added, changed and removed since the last frame, and readies the culls' buffers for the frame.
  pub fn prepare_draws(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut FrameEncoder) {
    self.geometry.flush(device, queue);
    self.cull.prepare(device, encoder);
  }

  /// The bounding sphere of every place that sways, and how far it reaches from its foot, weighted by its rigidity.
  pub fn list_swaying(&self) -> &[(Vec4, f32)] {
    &self.swaying
  }

  /// How far along `direction`, normalized, the level's sectors stand: the nearest and the farthest any reaches, none
  /// while it holds no sector.
  pub fn get_span(&self, direction: Vec3) -> Option<(f32, f32)> {
    self
      .sectors
      .as_slice()
      .iter()
      .map(|sector| {
        let along: f32 = sector.sphere.truncate().dot(direction);

        (along - sector.sphere.w, along + sector.sphere.w)
      })
      .reduce(|(nearest, farthest), (near, far)| (nearest.min(near), farthest.max(far)))
  }

  /// What the wind's amplitude is multiplied by to give the farthest any tree leans, in metres.
  pub fn get_sway_reach(&self) -> f32 {
    self.sway_reach
  }

  /// What of the scene drawn by passes of its own may be seen through a view's frustum `planes`.
  pub fn get_sight(&self, planes: &[Vec4; 6]) -> StaticSight {
    let is_in_view = |sphere: &Vec4| {
      planes
        .iter()
        .all(|plane| plane.truncate().dot(sphere.truncate()) + plane.w >= -sphere.w)
    };
    let sectors = self
      .sectors
      .as_slice()
      .iter()
      .flat_map(|sector| sector.sights.iter())
      .map(|(sphere, sight)| (sphere, *sight));
    let objects = self.objects.as_slice().iter().filter_map(|object| {
      let model: &StaticModelProxy = self.models.get(object.model)?;

      (!model.sight.is_empty()).then_some((&object.sphere, model.sight))
    });
    let mut seen: StaticSight = StaticSight::default();

    for (sphere, sight) in sectors.chain(objects) {
      if (seen | sight) != seen && is_in_view(sphere) {
        seen |= sight;

        if seen == StaticSight::ALL {
          break;
        }
      }
    }

    seen
  }

  /// The spawned objects with composited surfaces and the level's strictly sorted composited runs, which a view draws
  /// back to front itself.
  pub fn list_sorted_places(&self) -> impl Iterator<Item = StaticSortedPlace<'_>> {
    let objects = self.objects.as_slice().iter().filter_map(|object| {
      let model: &StaticModelProxy = self.models.get(object.model)?;

      (!model.sorted.is_empty()).then(|| StaticSortedPlace {
        sphere: object.sphere,
        place: object.place.get_offset(),
        layout: StaticLayout::Model,
        clusters: &model.sorted,
      })
    });
    let runs = self
      .sectors
      .as_slice()
      .iter()
      .flat_map(|sector| sector.sorted_runs.iter())
      .map(|run| StaticSortedPlace {
        sphere: run.sphere,
        place: run.place.unwrap_or(self.geometry.get_origin()),
        layout: run.layout,
        clusters: std::slice::from_ref(&run.clusters),
      });

    objects.chain(runs)
  }

  fn find_sector(&self, sector: u32) -> Option<&StaticSectorProxy> {
    self.sectors.as_slice().iter().find(|it| it.sector == sector)
  }

  /// Puts an instanced group: its geometry once, a slot per progressive band, a place per instance and a row per
  /// instance and band.
  #[allow(clippy::too_many_arguments)]
  fn put_instances(
    &mut self,
    (writer, swaying): (&mut StaticSceneWriter, &mut Vec<(Vec4, f32)>),
    (sector, mesh): (u32, u32),
    group: &SectorInstanceGroup,
    buffer: &[u8],
    descriptors: &[XraySurfaceDescriptor],
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) {
    let geometry: &SectorGeometry = &group.geometry;

    if geometry.index_count == 0 || group.instance_count == 0 {
      return;
    }

    let Some(StaticSurfaceUse { surface, class, passes }) = self.surfaces.resolve(
      writer,
      Self::level_surface(&group.surface, descriptors),
      textures,
      source,
    ) else {
      return;
    };
    let layout: StaticLayout = if geometry.uv_components >= 4 {
      StaticLayout::Tree
    } else {
      StaticLayout::Baked
    };
    let batch: StaticBatch = StaticBatch { layout, class };
    let sight: StaticSight = StaticSight::of(class, passes);
    let base: StaticGeometryBase = Self::put_geometry(writer, layout, geometry, buffer);
    let info: StaticSlotInfo = StaticSlotInfo {
      sector,
      shader_id: group.surface.shader_id,
      mesh: Some(mesh),
      first_place: writer.places.len() as u32,
    };
    let (bands, windows): (Vec<(u32, u32)>, u32) = match &group.progressive {
      Some(progressive) if !progressive.bands.is_empty() => (
        progressive.bands.iter().map(|band| (band.start, band.count)).collect(),
        progressive.windows.max(1),
      ),
      _ => (vec![(0, geometry.index_count)], 1),
    };
    let (band_slots, local): (Vec<(u32, u32)>, Vec4) = Self::put_bands(
      writer,
      (&base, geometry, buffer),
      &bands,
      (batch, surface, passes),
      info,
    );
    let transforms: Vec<f32> = read_pods(buffer, &group.transforms);
    let hemi: Vec<f32> = read_pods(buffer, &group.hemi);
    // Each place's impostor, in the sector's own numbering; a tree no impostor stands for has none.
    let lods: Vec<i32> = group
      .impostors
      .as_ref()
      .map_or_else(Vec::new, |it| read_pods(buffer, it));
    let impostors: u32 = writer.impostors.len() as u32;
    let instances: usize = (group.instance_count as usize).min(transforms.len() / 16);
    // Traced at its whole detail, unswayed: a reflection shows neither the coarser bands nor the sway's centimetres.
    let mut traced: Option<StaticTraceGroup> = class.is_deferred().then(|| StaticTraceGroup {
      geometry: StaticTraceGeometry {
        layout,
        first_index: base.index_start + bands[0].0,
        index_count: bands[0].1,
        vertex_start: base.vertex_start,
        vertex_count: geometry.vertex_count,
        surface,
        is_cut_out: class == StaticClass::CutOut,
      },
      first_place: writer.places.len() as u32,
      transforms: Vec::with_capacity(instances),
    });
    let sway_reach: f32 = if layout == StaticLayout::Tree {
      StaticVertexWords::measure_sway_reach(geometry, buffer)
    } else {
      0.0
    };

    for instance in 0..instances {
      let matrix: Mat4 = Mat4::from_cols_slice(&transforms[instance * 16..][..16]);
      let scale: f32 = Self::to_scale(&matrix);
      let (hemi_scale, hemi_bias): (f32, f32) = (
        hemi.get(instance * 2).copied().unwrap_or(1.0),
        hemi.get(instance * 2 + 1).copied().unwrap_or(0.0),
      );
      let place: u32 = writer.places.len() as u32;
      let center: Vec3 = matrix.transform_point3(local.truncate());

      if sway_reach > 0.0 {
        swaying.push((center.extend(local.w * scale), sway_reach * scale));
      }

      let lod: u32 = lods
        .get(instance)
        .and_then(|it| u32::try_from(*it).ok())
        .filter(|it| *it < impostors)
        .unwrap_or(StaticRow::NO_LOD);

      if let Some(traced) = &mut traced {
        traced.transforms.push(StaticTraceGroup::to_transform(&matrix));
      }

      if !sight.is_empty() {
        writer.sights.push((center.extend(local.w * scale), sight));
      }

      // A strictly sorted mesh is drawn place by place back to front, at its whole detail.
      if class == StaticClass::Sorted
        && let Some((slot, clusters)) = band_slots.first()
      {
        writer.sorted_runs.push(StaticSortedRun {
          sphere: center.extend(local.w * scale),
          place: Some(place),
          layout,
          clusters: (writer.slots[*slot as usize].first_cluster, *clusters),
        });
      }

      writer.places.push(StaticPlace {
        transform: matrix,
        info: Vec4::new(
          hemi_scale,
          hemi_bias,
          if lod == StaticRow::NO_LOD { -1.0 } else { lod as f32 },
          scale,
        ),
        cube: UVec4::ZERO,
        skin: UVec4::ZERO,
      });

      for (band, (slot, clusters)) in band_slots.iter().enumerate() {
        writer.rows.push(StaticRow {
          sphere: center.extend(local.w * scale),
          place,
          slot: *slot,
          lod,
          band: StaticRow::pack_band(band as u32, band_slots.len() as u32, windows),
        });
        for list in batch.list_draw_lists(passes) {
          writer.capacities[list] += clusters;
        }
      }
    }

    writer
      .trace_groups
      .extend(traced.filter(|group| !group.transforms.is_empty()));
  }

  /// Puts a sector's impostors in its own order, each with its run's surface; one no run dresses is never drawn.
  fn put_impostors(
    &mut self,
    writer: &mut StaticSceneWriter,
    impostors: &SectorImpostors,
    buffer: &[u8],
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) {
    let spheres: Vec<Vec4> = read_pods(buffer, &impostors.spheres);
    let factors: Vec<f32> = read_pods(buffer, &impostors.factors);
    let corners: Vec<[f32; SectorImpostors::FLOATS_PER_CORNER]> = read_pods(buffer, &impostors.corners);
    let normals: Vec<Vec4> = read_pods(buffer, &impostors.normals);
    let count: usize = spheres
      .len()
      .min(factors.len())
      .min(corners.len() / StaticImpostor::CORNERS)
      .min(normals.len() / StaticImpostor::FACETS);
    let mut dressed: Vec<Option<(u32, u16)>> = vec![None; count];

    for group in &impostors.groups {
      let row: u32 = self.surfaces.resolve_impostor(writer, &group.surface, textures, source);

      for index in (group.start as usize..).take(group.count as usize) {
        if let Some(it) = dressed.get_mut(index) {
          *it = Some((row, group.surface.shader_id));
        }
      }
    }

    for (index, dressing) in dressed.iter().enumerate() {
      let mut sphere: Vec4 = spheres[index];

      if dressing.is_none() {
        sphere.w = -1.0;
      }

      let mut facets: [Vec4; StaticImpostor::FACETS] = [Vec4::ZERO; StaticImpostor::FACETS];

      facets.copy_from_slice(&normals[index * StaticImpostor::FACETS..][..StaticImpostor::FACETS]);
      writer.impostors.push(StaticImpostor {
        sphere,
        normals: facets,
        factor: factors[index],
        surface: dressing.map_or(0, |it| it.0),
        _pad: [0; 2],
      });

      for corner in &corners[index * StaticImpostor::CORNERS..][..StaticImpostor::CORNERS] {
        // Position, atlas coordinate, then the hemisphere and sun terms.
        writer
          .corners
          .push(Vec4::new(corner[0], corner[1], corner[2], corner[5]));
        writer.corners.push(Vec4::new(corner[3], corner[4], corner[6], 0.0));
      }

      writer.impostor_shaders.push(dressing.map_or(0, |it| it.1));
    }
  }

  /// The largest scale a matrix stands anything at, which a sphere it moves grows by.
  fn to_scale(matrix: &Mat4) -> f32 {
    [matrix.x_axis, matrix.y_axis, matrix.z_axis]
      .iter()
      .map(|axis| axis.truncate().length())
      .fold(0.0, f32::max)
  }

  fn put_geometry(
    writer: &mut StaticSceneWriter,
    layout: StaticLayout,
    geometry: &SectorGeometry,
    buffer: &[u8],
  ) -> StaticGeometryBase {
    writer.put_words(
      layout,
      &StaticVertexWords::pack_vertex_words(layout, geometry, buffer),
      &read_pods::<u32>(buffer, &geometry.indices),
    )
  }

  /// A spawned object's lighting as a place holds it: each cube face a byte, `+x +y +z -x` in `x`, `-y -z` in `y`, the
  /// sky share's bits in `z`, and `w` marking it.
  fn pack_cube((faces, sky): ([f32; 6], f32)) -> UVec4 {
    let bytes: [u32; 6] = faces.map(|face| (face.clamp(0.0, 1.0) * 255.0).round() as u32);

    UVec4::new(
      bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24),
      bytes[4] | (bytes[5] << 8),
      sky.to_bits(),
      1,
    )
  }

  /// A level's surface as `resolve_surface` takes one: its shader table entry, dressed by that entry's descriptor.
  fn level_surface<'a>(
    surface: &'a SectorSurface,
    descriptors: &'a [XraySurfaceDescriptor],
  ) -> (StaticSurfaceKey, &'a SectorSurface, Option<&'a XraySurfaceDescriptor>) {
    (
      StaticSurfaceKey::Level(surface.shader_id),
      surface,
      descriptors.get(surface.shader_id as usize),
    )
  }

  /// The clusters covering an index range, as `(first index, triangles, sphere)`: the packer's own where they tile it
  /// exactly, else the range cut in runs of `CLUSTER_TRIANGLES` with spheres taken from their vertices.
  /// Puts a slot per band (`bands`, each a first index and a count) of an instanced group's geometry. Answers the band
  /// slots with their clusters, and the bands' bounding sphere in the mesh's own space.
  fn put_bands(
    writer: &mut StaticSceneWriter,
    (base, geometry, buffer): (&StaticGeometryBase, &SectorGeometry, &[u8]),
    bands: &[(u32, u32)],
    (batch, surface, passes): (StaticBatch, u32, u32),
    info: StaticSlotInfo,
  ) -> (Vec<(u32, u32)>, Vec4) {
    let band_slots: Vec<(u32, u32)> = bands
      .iter()
      .map(|range| {
        writer.put_slot(
          base,
          Self::cut_clusters(geometry, buffer, range.0, range.1),
          (StaticSlot::LISTED, batch, surface, passes),
          info,
        )
      })
      .collect();
    let local: Vec4 = Self::bound_spheres(
      &writer.spheres[writer.spheres.len() - band_slots.iter().map(|it| it.1 as usize).sum::<usize>()..],
    );

    (band_slots, local)
  }

  fn cut_clusters(geometry: &SectorGeometry, buffer: &[u8], start: u32, count: u32) -> Vec<(u32, u32, Vec4)> {
    let ranges: Vec<[u32; 4]> = read_pods(buffer, &geometry.clusters.ranges);
    let spheres: Vec<Vec4> = read_pods(buffer, &geometry.clusters.spheres);
    let end: u32 = start + count;
    let packed: Vec<(u32, u32, Vec4)> = ranges
      .iter()
      .zip(&spheres)
      .filter(|(range, _)| range[0] >= start && range[0] < end)
      .map(|(range, sphere)| (range[0], range[1], *sphere))
      .collect();

    if packed.iter().map(|(_, triangles, _)| triangles * 3).sum::<u32>() == count {
      return packed;
    }

    let indices: Vec<u32> = read_pods(buffer, &geometry.indices);
    let positions: Vec<[f32; 3]> = read_pods(buffer, &geometry.positions);

    (start..end)
      .step_by(StaticGeometry::CLUSTER_VERTICES as usize)
      .map(|first| {
        let triangles: u32 = ((end - first) / 3).min(StaticGeometry::CLUSTER_TRIANGLES);
        let points: Vec<Vec3> = indices
          .iter()
          .skip(first as usize)
          .take((triangles * 3) as usize)
          .filter_map(|index| positions.get(*index as usize).copied().map(Vec3::from_array))
          .collect();

        (first, triangles, Self::bound_points(&points))
      })
      .collect()
  }

  /// A sphere holding every point: centred on their box, as far out as the farthest.
  fn bound_points(points: &[Vec3]) -> Vec4 {
    let (low, high): (Vec3, Vec3) = points.iter().fold((Vec3::MAX, Vec3::MIN), |(low, high), point| {
      (low.min(*point), high.max(*point))
    });
    let center: Vec3 = (low + high) * 0.5;
    let radius: f32 = points.iter().map(|point| point.distance(center)).fold(0.0, f32::max);

    if points.is_empty() {
      Vec4::new(0.0, 0.0, 0.0, -1.0)
    } else {
      center.extend(radius)
    }
  }

  /// A sphere holding every sphere: centred on their box, as far out as the farthest edge.
  fn bound_spheres(spheres: &[Vec4]) -> Vec4 {
    let (low, high): (Vec3, Vec3) = spheres.iter().fold((Vec3::MAX, Vec3::MIN), |(low, high), sphere| {
      (
        low.min(sphere.truncate() - sphere.w),
        high.max(sphere.truncate() + sphere.w),
      )
    });
    let center: Vec3 = (low + high) * 0.5;
    let radius: f32 = spheres
      .iter()
      .map(|sphere| sphere.truncate().distance(center) + sphere.w)
      .fold(0.0, f32::max);

    if spheres.is_empty() {
      Vec4::new(0.0, 0.0, 0.0, -1.0)
    } else {
      center.extend(radius)
    }
  }
}
