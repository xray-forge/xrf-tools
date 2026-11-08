use glam::{Mat4, UVec4, Vec3, Vec4};
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer_core::{ProxyHandle, Span};
use xrf_visual::{SectorGeometry, SectorInstanceGroup, SectorPackage};

use crate::host::level::spawn::model::static_model_place::StaticModelPlace;
use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::draw::slot::static_row::StaticRow;
use crate::scene::static_scene::draw::slot::static_slot::StaticSlot;
use crate::scene::static_scene::draw::slot::static_slot_draw::StaticSlotDraw;
use crate::scene::static_scene::draw::slot::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::draw::static_batch::StaticBatch;
use crate::scene::static_scene::draw::static_class::StaticClass;
use crate::scene::static_scene::draw::static_sight::StaticSight;
use crate::scene::static_scene::geometry::geometry_clusters::GeometryClusters;
use crate::scene::static_scene::geometry::static_geometry_base::StaticGeometryBase;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::geometry::vertex::static_vertex_words::StaticVertexWords;
use crate::scene::static_scene::item::model::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::item::object::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::item::static_item_context::StaticItemContext;
use crate::scene::static_scene::item::static_place::StaticPlace;
use crate::scene::static_scene::item::static_scene_writer::StaticSceneWriter;
use crate::scene::static_scene::item::static_sorted_run::StaticSortedRun;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::surface::static_surface_use::StaticSurfaceUse;
use crate::scene::static_scene::trace::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::trace::static_trace_group::StaticTraceGroup;

impl StaticScene {
  /// Stands a spawned object as a model in the scene at the handle its poster allocated: a place, its bone matrices in
  /// the bind pose where the model is skinned, and a row a slot of the model.
  ///
  /// # Errors
  ///
  /// Returns an error for a handle that holds an object already, a model no longer in the scene, or when a buffer would
  /// outgrow what the device allows; the scene is left as it was.
  pub(crate) fn add_object(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    handle: ProxyHandle<StaticObjectProxy>,
    model: ProxyHandle<StaticModelProxy>,
    place: &StaticModelPlace,
  ) -> XrfResult {
    if self.objects.is_held(handle) {
      return Err(XrfError::new_invalid_error(format!(
        "Object posted at {handle:?}, which holds one already"
      )));
    }

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
    let mut capacities: [u32; StaticBatch::LIST_COUNT] = [0; StaticBatch::LIST_COUNT];

    for (_, clusters, batch, passes) in &slots {
      for list in batch.list_draw_lists(*passes) {
        capacities[list] += clusters;
      }
    }

    self.cull.check_capacities(device, &capacities)?;

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
        // A model's normals carry no hemisphere, so the bias is its whole hemisphere where no cube lights it: half, as
        // the engine lights an object before its first estimate (`CROS_impl`'s `hemi_value` and `hemi_smooth`).
        info: Vec4::new(1.0, 0.5, -1.0, scale),
        cube: place.lighting.map_or(UVec4::ZERO, Self::pack_cube),
        skin: match (&bones, skin) {
          (Some((rows, bones)), Some((links, _))) => UVec4::new(rows.get_offset(), links, vertex_start, *bones),
          _ => UVec4::ZERO,
        },
      }),
    );

    let rows: Vec<ProxyHandle<StaticRow>> = slots
      .iter()
      .map(|(slot, ..)| {
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
  pub(crate) fn remove_object(&mut self, handle: ProxyHandle<StaticObjectProxy>) -> bool {
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

  /// Puts a sector's instanced group, its `mesh`: its geometry once, a slot per progressive band, a place per instance
  /// and a row per instance and band, each place's sway noted into `swaying`.
  pub(super) fn put_instances(
    &mut self,
    writer: &mut StaticSceneWriter,
    swaying: &mut Vec<(Vec4, f32)>,
    package: &SectorPackage,
    mesh: u32,
    descriptors: &[XraySurfaceDescriptor],
    context: &mut StaticItemContext<'_>,
  ) {
    let group: &SectorInstanceGroup = &package.description.instances[mesh as usize];
    let (sector, buffer): (u32, &[u8]) = (package.description.sector, &package.buffer);
    let geometry: &SectorGeometry = &group.geometry;

    if geometry.index_count == 0 || group.instance_count == 0 {
      return;
    }

    let Some(StaticSurfaceUse { surface, class, passes }) = self.surfaces.resolve(
      writer,
      Self::level_surface(&group.surface, descriptors),
      context.textures,
      context.source,
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
      &base,
      &GeometryClusters::new(geometry, buffer),
      &bands,
      StaticSlotDraw {
        kind: StaticSlot::LISTED,
        batch,
        surface,
        passes,
      },
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
}
