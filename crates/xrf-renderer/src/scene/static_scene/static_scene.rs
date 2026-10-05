use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use glam::{Mat4, UVec4, Vec3, Vec4};
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer_core::{ProxyHandle, ProxyStore, Span, SpanBuffer, SpanLane, StoreMirror};
use xrf_visual::{SectorGeometry, SectorImpostors, SectorInstanceGroup, SectorPackage, SectorSurface, VisualClusters};

use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::contract::render_static_report::RenderStaticReport;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_cluster::StaticCluster;
use crate::scene::static_scene::static_geometry_base::StaticGeometryBase;
use crate::scene::static_scene::static_impostor::StaticImpostor;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_model::StaticModel;
use crate::scene::static_scene::static_model_place::StaticModelPlace;
use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_region::StaticRegion;
use crate::scene::static_scene::static_row::StaticRow;
use crate::scene::static_scene::static_scene_writer::StaticSceneWriter;
use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::static_selection::StaticSelection;
use crate::scene::static_scene::static_slot::StaticSlot;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::static_sorted_place::StaticSortedPlace;
use crate::scene::static_scene::static_spans::StaticSpans;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_build::{build_impostor_surface, build_static_surface};
use crate::scene::static_scene::static_surface_key::StaticSurfaceKey;
use crate::scene::static_scene::static_vertex_words::{measure_sway_reach, pack_vertex_words};
use crate::scene::texture::texture_cache::{MISSING_SLOT, TextureCache};

/// Clusters a spawned model's row holds at most: the row cull walks a row's clusters on one thread.
const MODEL_ROW_CLUSTERS: usize = 32;

/// Triangles a cluster holds at most.
const CLUSTER_TRIANGLES: u32 = VisualClusters::MAX_TRIANGLES;

/// A level's static geometry on the GPU, as sectors, spawned models and the objects standing as them are added and
/// removed: span buffers of vertices, indices, clusters, slots, places, impostors and skins each item takes a range of,
/// the rows a store keeps dense for the cull to walk, and the surfaces every item shares. A cull walks the clusters and
/// impostors whole, so a range is cleared as it is freed and reads as nothing.
pub struct StaticScene {
  pub words: [SpanBuffer; StaticLayout::COUNT],
  pub indices: SpanBuffer,
  /// The clusters, and their spheres in the `CLUSTER_SPHERES` lane.
  pub clusters: SpanBuffer,
  pub slots: SpanBuffer,
  pub places: SpanBuffer,
  /// Every place's rows, dense, for the row cull to walk.
  pub rows: StoreMirror,
  pub surfaces: GrowableBuffer,
  pub lists: GrowableBuffer,
  /// Clusters the early phase set aside, for the late phase to test again.
  pub candidates: GrowableBuffer,
  /// The impostors, and by lane their corners (two vectors a corner: its position and hemisphere term, then its atlas
  /// coordinate and sun term), what each one's level of detail draws this frame as the cull decided it, and the list of
  /// those drawn this frame.
  pub impostors: SpanBuffer,
  /// Skinned models' links, two words a vertex, and the objects' bone matrices, three rows a bone.
  pub skins: SpanBuffer,
  pub bones: SpanBuffer,
  pub regions: wgpu::Buffer,
  pub args: wgpu::Buffer,
  /// Each batch's late draw arguments, then the late phase's dispatch and the candidates' count.
  pub late: wgpu::Buffer,
  /// The late phase's dispatch, copied out of `late` so the dispatch reads no buffer it binds.
  pub late_dispatch: wgpu::Buffer,
  /// Every batch's draw arguments as a cull starts them, copied into a view's own before each cull of it in a frame.
  pub args_template: wgpu::Buffer,
  row_records: ProxyStore<StaticRow>,
  /// Place zero, where every single draw stands: its geometry is in renderer space already. Never freed.
  origin: Span,
  /// Entries the visible list and the candidates hold this frame.
  list_capacity: u32,
  /// Every batch's draw arguments as a cull starts them, which a shadow's own arguments start from too.
  initial_args: Vec<u32>,
  /// Each impostor surface's row, by its shader table entry.
  impostor_rows: HashMap<u16, u32>,
  /// Each surface's row and class, once asked for; `None` for one no static draw draws.
  surface_rows: HashMap<StaticSurfaceKey, Option<(u32, StaticClass)>>,
  /// Every texture slot the scene's surfaces sample, in the order they were first asked for.
  pub texture_slots: BTreeSet<u32>,
  /// Every environment slot its environment-mapped surfaces sample.
  pub environment_slots: BTreeSet<u32>,
  surface_count: u32,
  /// Entries of the visible list each batch may need: its clusters, each counted once per place drawing it.
  capacities: [u32; StaticBatch::COUNT],
  sectors: ProxyStore<StaticSectorProxy>,
  models: ProxyStore<StaticModelProxy>,
  objects: ProxyStore<StaticObjectProxy>,
  /// Each spawned object in the scene, by its index, and by its place.
  object_handles: HashMap<u32, ProxyHandle<StaticObjectProxy>>,
  place_objects: HashMap<u32, ProxyHandle<StaticObjectProxy>>,
  /// The bounding sphere of every place that sways and its reach, which a light's shadow looks for in its range.
  swaying: Vec<(Vec4, f32)>,
  /// The farthest any swaying vertex reaches from its tree's foot, weighted by its rigidity and scaled by its place.
  sway_reach: f32,
  /// Items added and removed, which a view drawn with another count is drawn again for.
  contents: usize,
}

impl StaticScene {
  /// Vertices one cluster's draw pulls: its triangles' corners, a short cluster's tail collapsed.
  pub const CLUSTER_VERTICES: u32 = CLUSTER_TRIANGLES * 3;

  /// Where the cull's counts follow every batch's draw arguments in their buffer.
  pub const STATS_OFFSET: u64 = (StaticBatch::COUNT * 16) as u64;

  /// Where the impostors' draw arguments follow the cull's counts.
  pub const IMPOSTOR_ARGS_OFFSET: u64 = Self::STATS_OFFSET + 16;

  /// Where the late phase's dispatch follows every batch's late draw arguments in theirs.
  pub const LATE_DISPATCH_OFFSET: u64 = (StaticBatch::COUNT * 16) as u64;

  /// The clusters' lane of their spheres.
  pub const CLUSTER_SPHERES: usize = 1;

  /// The impostors' lanes: their corners, their terms this frame, and the list of those drawn.
  pub const IMPOSTOR_CORNERS: usize = 1;
  pub const IMPOSTOR_TERMS: usize = 2;
  pub const IMPOSTOR_LIST: usize = 3;

  /// Vectors an impostor's corners take: two a corner.
  pub const CORNER_VECTORS: usize = StaticImpostor::CORNERS * 2;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let storage: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE;
    let words: u64 = u64::from(StaticLayout::STRIDE) * 4;
    let mut places: SpanBuffer = SpanBuffer::new(
      device,
      "static places",
      size_of::<StaticPlace>() as u64,
      storage,
      1 << 10,
    );
    let origin: Span = places
      .allocate(device, queue, 1)
      .expect("A fresh span buffer holds its first element");

    assert_eq!(origin.get_offset(), 0, "The origin place is the first");
    places.write(&origin, 0, bytemuck::bytes_of(&StaticPlace::default()));

    let mut scene: Self = Self {
      words: [
        SpanBuffer::new(device, "static baked vertices", words, storage, 1 << 14),
        SpanBuffer::new(device, "static tree vertices", words, storage, 1 << 14),
        SpanBuffer::new(device, "static model vertices", words, storage, 1 << 14),
      ],
      indices: SpanBuffer::new(device, "static indices", 4, storage, 1 << 16),
      clusters: SpanBuffer::with_lanes(
        device,
        &[
          SpanLane::new("static clusters", size_of::<StaticCluster>() as u64),
          SpanLane::new("static spheres", size_of::<Vec4>() as u64),
        ],
        storage,
        1 << 12,
      ),
      slots: SpanBuffer::new(device, "static slots", size_of::<StaticSlot>() as u64, storage, 1 << 10),
      places,
      rows: StoreMirror::new(device, "static rows", size_of::<StaticRow>() as u64, storage),
      surfaces: GrowableBuffer::new(device, "static surfaces", storage),
      lists: GrowableBuffer::new(device, "static lists", storage),
      candidates: GrowableBuffer::new(device, "static candidates", storage),
      impostors: SpanBuffer::with_lanes(
        device,
        &[
          SpanLane::new("static impostors", size_of::<StaticImpostor>() as u64),
          SpanLane::new(
            "static impostor corners",
            (Self::CORNER_VECTORS * size_of::<Vec4>()) as u64,
          ),
          SpanLane::new("static impostor terms", 16),
          SpanLane::new("static impostor list", 4),
        ],
        storage,
        1 << 8,
      ),
      skins: SpanBuffer::new(device, "static model skins", 8, storage, 1 << 14),
      bones: SpanBuffer::new(device, "static bones", size_of::<Vec4>() as u64, storage, 1 << 12),
      regions: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("static regions"),
        size: (StaticBatch::COUNT * size_of::<StaticRegion>()) as u64,
        usage: storage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("static draw arguments"),
        size: Self::IMPOSTOR_ARGS_OFFSET + 16,
        usage: storage | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      late: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("static late draw arguments"),
        size: Self::LATE_DISPATCH_OFFSET + 16,
        usage: storage | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }),
      args_template: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("static draw arguments template"),
        size: Self::IMPOSTOR_ARGS_OFFSET + 16,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      late_dispatch: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("static late dispatch"),
        size: 12,
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      row_records: ProxyStore::new(),
      origin,
      list_capacity: 0,
      initial_args: Vec::new(),
      impostor_rows: HashMap::new(),
      surface_rows: HashMap::new(),
      texture_slots: BTreeSet::new(),
      environment_slots: BTreeSet::new(),
      surface_count: 0,
      capacities: [0; StaticBatch::COUNT],
      sectors: ProxyStore::new(),
      models: ProxyStore::new(),
      objects: ProxyStore::new(),
      object_handles: HashMap::new(),
      place_objects: HashMap::new(),
      swaying: Vec::new(),
      sway_reach: 0.0,
      contents: 0,
    };
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

    // Surface row zero is never named, so a slot left at zero still reads a valid row.
    scene.surfaces.append(
      device,
      queue,
      &mut encoder,
      bytemuck::bytes_of(&StaticSurface::default()),
    );
    scene.surface_count = 1;
    queue.submit([encoder.finish()]);

    scene
  }

  /// How full the scene's pools are: what a report of its static draws starts from.
  pub fn get_pools(&self) -> RenderStaticReport {
    RenderStaticReport {
      slots: to_pool_use(&self.slots),
      places: to_pool_use(&self.places),
      rows: RenderPoolUse {
        used: self.row_records.len() as u32,
        capacity: self.rows.get_capacity(),
      },
      lods: to_pool_use(&self.impostors),
      clusters: to_pool_use(&self.clusters),
      surface_list: RenderPoolUse {
        used: 0,
        capacity: self.list_capacity,
      },
      ..Default::default()
    }
  }

  /// Clusters the cull walks: the whole buffer, a range nothing holds reading as an empty cluster.
  pub fn get_cluster_count(&self) -> u32 {
    self.clusters.get_capacity()
  }

  /// Rows the cull walks, as of the last `prepare_draws`.
  pub fn get_row_count(&self) -> u32 {
    self.rows.get_count()
  }

  /// Impostors the cull walks: the whole buffer, a range nothing holds reading as no impostor.
  pub fn get_impostor_count(&self) -> u32 {
    self.impostors.get_capacity()
  }

  /// Sectors in the scene.
  pub fn get_sector_count(&self) -> usize {
    self.sectors.len()
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
    self.get_object_at(place).map(|object| object.object)
  }

  /// Whether a spawned object stands in the scene.
  pub fn has_object(&self, object: u32) -> bool {
    self.object_handles.contains_key(&object)
  }

  /// A spawned object's bounding sphere in renderer space, while it stands in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.get_object(object).map(|object| object.sphere)
  }

  /// A spawned object's box as it stands: its model's box in its own space, and where it stands it.
  pub fn get_object_box(&self, object: u32) -> Option<(Mat4, [Vec3; 2])> {
    let object: &StaticObjectProxy = self.get_object(object)?;

    Some((object.transform, self.models.get(object.model)?.bounds))
  }

  /// Where a spawned object stands, while it stands in the scene.
  pub fn get_object_transform(&self, object: u32) -> Option<Mat4> {
    self.get_object(object).map(|object| object.transform)
  }

  /// What a selection marks, once what it names is in the scene: a spawned object's place; a mesh's place; or the
  /// runs of clusters drawing a sector's baked geometry of one shader table entry.
  pub fn resolve_selection(&self, target: &RenderSelectionTarget) -> Option<StaticSelection> {
    match *target {
      RenderSelectionTarget::Spawn { object } => Some(StaticSelection {
        place: Some(self.get_object(object)?.place.get_offset()),
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
  pub fn write_pose(&mut self, object: u32, current: &[[Vec4; 3]], previous: &[[Vec4; 3]]) {
    let Some((span, bones)) = self
      .object_handles
      .get(&object)
      .and_then(|handle| self.objects.get(*handle))
      .and_then(|object| object.bones.as_ref())
    else {
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

    self.bones.write(span, 0, bytemuck::cast_slice(&rows));
  }

  /// Bytes of every pack put into the scene.
  pub fn get_bytes(&self) -> u64 {
    self.sectors.as_slice().iter().map(|sector| sector.bytes).sum()
  }

  /// A generation of every buffer a bind group holds, which changes whenever one of them was replaced.
  pub fn get_generation(&self) -> u64 {
    self
      .list_spans()
      .iter()
      .map(|buffer| buffer.get_generation())
      .sum::<u64>()
      + self.rows.get_generation()
      + self.surfaces.get_generation()
      + self.lists.get_generation()
      + self.candidates.get_generation()
  }

  /// Bytes the scene's growing buffers hold on the GPU, which is what grows with a level or a model.
  pub fn get_buffer_bytes(&self) -> u64 {
    self.list_spans().iter().map(|buffer| buffer.get_bytes()).sum::<u64>()
      + self.rows.get_bytes()
      + self.surfaces.get_capacity()
      + self.lists.get_capacity()
      + self.candidates.get_capacity()
  }

  /// Puts one packed sector into the scene: its baked sections as single draws, its tree groups as rows.
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
    package: &SectorPackage,
  ) -> XrfResult<ProxyHandle<StaticSectorProxy>> {
    let mut writer: StaticSceneWriter = StaticSceneWriter::default();
    let mut swaying: Vec<(Vec4, f32)> = Vec::new();
    let description = &package.description;
    let buffer: &[u8] = &package.buffer;

    if !description.sections.is_empty() && description.geometry.index_count > 0 {
      let base: StaticGeometryBase = put_geometry(&mut writer, StaticLayout::Baked, &description.geometry, buffer);

      for section in &description.sections {
        let Some((surface, class)) = self.resolve_surface(
          &mut writer,
          level_surface(&section.surface, descriptors),
          textures,
          source,
        ) else {
          continue;
        };
        let batch: StaticBatch = StaticBatch {
          layout: StaticLayout::Baked,
          class,
        };
        let (_, clusters): (u32, u32) = writer.put_slot(
          &base,
          cut_clusters(&description.geometry, buffer, section.draw.start, section.draw.count),
          (StaticSlot::SINGLE, batch, surface),
          StaticSlotInfo {
            sector: description.sector,
            shader_id: section.surface.shader_id,
            mesh: None,
            first_place: 0,
          },
        );

        writer.capacities[batch.get_index() as usize] += clusters;
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

    self.put_surfaces(device, queue, encoder, &writer);

    let spans: StaticSpans = self.allocate_spans(device, queue, &writer)?;

    writer.relocate(&spans);
    self.write_spans(&spans, &writer);

    let rows: Vec<ProxyHandle<StaticRow>> = writer.rows.iter().map(|row| self.row_records.add(*row)).collect();

    self.add_capacities(&writer.capacities);
    self.sway_reach = swaying.iter().map(|(_, reach)| *reach).fold(self.sway_reach, f32::max);
    self.swaying.extend_from_slice(&swaying);
    self.contents += 1;

    Ok(self.sectors.add(StaticSectorProxy {
      sector: description.sector,
      bytes: buffer.len() as u64,
      spans,
      rows,
      capacities: writer.capacities,
      slot_infos: writer.slot_infos,
      cluster_slots: writer.cluster_slots,
      impostor_shaders: writer.impostor_shaders,
      swaying,
    }))
  }

  /// Takes a sector out of the scene, its ranges cleared and freed; false for one already gone.
  pub fn remove_sector(&mut self, handle: ProxyHandle<StaticSectorProxy>) -> bool {
    let Some(sector) = self.sectors.remove(handle) else {
      return false;
    };

    for row in sector.rows {
      self.row_records.remove(row);
    }

    self.remove_capacities(&sector.capacities);
    self.release_spans(sector.spans);
    self.swaying = self
      .sectors
      .as_slice()
      .iter()
      .flat_map(|sector| sector.swaying.iter().copied())
      .collect();
    self.sway_reach = self.swaying.iter().map(|(_, reach)| *reach).fold(0.0, f32::max);
    self.contents += 1;

    true
  }

  /// Puts a spawned model into the scene: its geometry once and a slot a part, which each object standing as it adds a
  /// row of.
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
    model: &StaticModel,
  ) -> XrfResult<ProxyHandle<StaticModelProxy>> {
    let mut writer: StaticSceneWriter = StaticSceneWriter::default();
    let base: StaticGeometryBase = writer.put_words(StaticLayout::Model, &model.words, &model.indices);
    let mut slots: Vec<(u32, u32, StaticBatch)> = Vec::new();
    // The composited parts' clusters, which each object lists to be drawn sorted.
    let mut sorted: Vec<(u32, u32)> = Vec::new();

    for part in &model.parts {
      let key: StaticSurfaceKey = StaticSurfaceKey::Model {
        shader: part.surface.shader_name.clone(),
        texture: part.surface.texture_name.clone(),
      };
      let Some((surface, class)) = self.resolve_surface(
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

      // A row's clusters are culled one after another by one thread, so a large part is cut into several slots.
      for clusters in part.clusters.chunks(MODEL_ROW_CLUSTERS) {
        let first_cluster: u32 = writer.clusters.len() as u32;
        let (slot, count): (u32, u32) = writer.put_slot(
          &base,
          clusters.to_vec(),
          (StaticSlot::LISTED, batch, surface),
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

        slots.push((slot, count, batch));
      }
    }

    if let Some(skin) = &model.skin {
      writer.skins.extend_from_slice(&skin.links);
    }

    self.put_surfaces(device, queue, encoder, &writer);

    let spans: StaticSpans = self.allocate_spans(device, queue, &writer)?;
    let first_slot: u32 = StaticSpans::get_offset(&spans.slots);
    let first_cluster: u32 = StaticSpans::get_offset(&spans.clusters);

    writer.relocate(&spans);
    self.write_spans(&spans, &writer);
    self.contents += 1;

    Ok(
      self.models.add(StaticModelProxy {
        slots: slots
          .into_iter()
          .map(|(slot, count, batch)| (first_slot + slot, count, batch))
          .collect(),
        sorted: sorted
          .into_iter()
          .map(|(first, count)| (first_cluster + first, count))
          .collect(),
        slot_infos: writer.slot_infos,
        cluster_slots: writer.cluster_slots,
        sphere: model.sphere,
        bounds: model.bounds,
        vertex_start: StaticSpans::get_offset(&spans.words[StaticLayout::Model.get_index()]) + base.vertex_start,
        skin: model
          .skin
          .as_ref()
          .map(|skin| (StaticSpans::get_offset(&spans.skins), skin.bones)),
        spans,
      }),
    )
  }

  /// Takes a model out of the scene with every object standing as it, its ranges cleared and freed; false for one
  /// already gone.
  pub fn remove_model(&mut self, handle: ProxyHandle<StaticModelProxy>) -> bool {
    if !self.models.contains(handle) {
      return false;
    }

    let standing: Vec<ProxyHandle<StaticObjectProxy>> = self
      .objects
      .iter()
      .filter(|(_, object)| object.model == handle)
      .map(|(object, _)| object)
      .collect();

    for object in standing {
      self.remove_object(object);
    }

    if let Some(model) = self.models.remove(handle) {
      self.release_spans(model.spans);
    }

    self.contents += 1;

    true
  }

  /// Stands a spawned object as a model in the scene: a place, its bone matrices in the bind pose where the model is
  /// skinned, and a row a slot of the model.
  ///
  /// # Errors
  ///
  /// Returns an error for a model no longer in the scene, or when a buffer would outgrow what the device allows; the
  /// scene is left as it was.
  pub fn add_object(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    model: ProxyHandle<StaticModelProxy>,
    place: &StaticModelPlace,
  ) -> XrfResult<ProxyHandle<StaticObjectProxy>> {
    let Some(proxy) = self.models.get(model) else {
      return Err(XrfError::new_invalid_error(format!(
        "Spawned object {} stands as a model no longer in the scene",
        place.object
      )));
    };
    let slots: Vec<(u32, u32, StaticBatch)> = proxy.slots.clone();
    let (sphere, vertex_start, skin): (Vec4, u32, Option<(u32, u32)>) = (proxy.sphere, proxy.vertex_start, proxy.skin);
    let matrix: Mat4 = place.transform;
    let scale: f32 = to_scale(&matrix);
    let placed: Vec4 = matrix.transform_point3(sphere.truncate()).extend(sphere.w * scale);
    let span: Span = self.places.allocate(device, queue, 1)?;
    let bones: Option<(Span, u32)> = match skin {
      Some((_, bones)) => {
        let bones: u32 = bones.max(1);

        match self.bones.allocate(device, queue, bones * 2 * 3) {
          // This frame's matrices and the last's, both the bind pose until a pose is written.
          Ok(rows) => {
            let bind: Vec<[Vec4; 3]> = vec![[Vec4::X, Vec4::Y, Vec4::Z]; bones as usize * 2];

            self.bones.write(&rows, 0, bytemuck::cast_slice(&bind));

            Some((rows, bones))
          }
          Err(error) => {
            self.places.free(span);

            return Err(error);
          }
        }
      }
      None => None,
    };
    let index: u32 = span.get_offset();

    self.places.write(
      &span,
      0,
      bytemuck::bytes_of(&StaticPlace {
        transform: matrix,
        info: Vec4::new(1.0, 0.0, -1.0, scale),
        cube: place.lighting.map_or(UVec4::ZERO, pack_cube),
        skin: match (&bones, skin) {
          (Some((rows, bones)), Some((links, _))) => UVec4::new(rows.get_offset(), links, vertex_start, *bones),
          _ => UVec4::ZERO,
        },
      }),
    );

    let mut capacities: [u32; StaticBatch::COUNT] = [0; StaticBatch::COUNT];
    let rows: Vec<ProxyHandle<StaticRow>> = slots
      .iter()
      .map(|(slot, clusters, batch)| {
        capacities[batch.get_index() as usize] += clusters;

        self.row_records.add(StaticRow {
          sphere: placed,
          place: index,
          slot: *slot,
          lod: StaticRow::NO_LOD,
          band: StaticRow::pack_band(0, 1, 1),
        })
      })
      .collect();

    self.add_capacities(&capacities);
    self.contents += 1;

    let handle: ProxyHandle<StaticObjectProxy> = self.objects.add(StaticObjectProxy {
      object: place.object,
      model,
      place: span,
      bones,
      rows,
      capacities,
      transform: matrix,
      sphere: placed,
    });

    // An object stood again replaces where it stood.
    if let Some(replaced) = self.object_handles.insert(place.object, handle) {
      self.remove_object(replaced);
    }

    self.place_objects.insert(index, handle);

    Ok(handle)
  }

  /// Takes a spawned object out of the scene: its place, bone matrices and rows; false for one already gone.
  pub fn remove_object(&mut self, handle: ProxyHandle<StaticObjectProxy>) -> bool {
    let Some(object) = self.objects.remove(handle) else {
      return false;
    };

    for row in object.rows {
      self.row_records.remove(row);
    }

    if self.object_handles.get(&object.object) == Some(&handle) {
      self.object_handles.remove(&object.object);
    }

    self.place_objects.remove(&object.place.get_offset());
    self.remove_capacities(&object.capacities);
    self.places.free(object.place);

    if let Some((bones, _)) = object.bones {
      self.bones.free(bones);
    }

    self.contents += 1;

    true
  }

  /// Takes a spawned object out of the scene by its index; false for one not standing in it.
  pub fn remove_object_by_index(&mut self, object: u32) -> bool {
    self
      .object_handles
      .get(&object)
      .copied()
      .is_some_and(|handle| self.remove_object(handle))
  }

  /// Items added and removed so far, which a view drawn with another count is drawn again for.
  pub fn get_contents(&self) -> usize {
    self.contents
  }

  /// Uploads what was added, changed and removed since the last frame, and rewrites each batch's run of the visible
  /// list and its draw arguments empty, for a frame's cull to fill.
  pub fn prepare_draws(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder) {
    for buffer in self.list_spans_mut() {
      buffer.flush(queue);
    }

    self.rows.sync(device, queue, &mut self.row_records, |row| *row);

    let mut base: u32 = 0;
    let mut regions: Vec<StaticRegion> = Vec::with_capacity(StaticBatch::COUNT);
    let mut args: Vec<u32> = Vec::with_capacity(StaticBatch::COUNT * 4 + 4);
    let mut late: Vec<u32> = Vec::with_capacity(StaticBatch::COUNT * 4 + 4);

    for capacity in self.capacities {
      regions.push(StaticRegion { base, capacity });
      // Every visible cluster is an instance of one cluster's vertices, its entry found by its instance index; the
      // late draw's first instance is set once the early phase has counted its own.
      args.extend_from_slice(&[CLUSTER_TRIANGLES * 3, 0, 0, base]);
      late.extend_from_slice(&[CLUSTER_TRIANGLES * 3, 0, 0, base]);
      base += capacity;
    }

    // The cull's counts, then the impostors' draw: a quad an impostor.
    args.extend_from_slice(&[0; 4]);
    args.extend_from_slice(&[6, 0, 0, 0]);
    late.extend_from_slice(&[0, 1, 1, 0]);

    self.list_capacity = base;
    self.lists.reserve(device, encoder, (base.max(1) as u64) * 8);
    self.candidates.reserve(device, encoder, (base.max(1) as u64) * 8);
    queue.write_buffer(&self.regions, 0, bytemuck::cast_slice(&regions));
    queue.write_buffer(&self.args, 0, bytemuck::cast_slice(&args));
    queue.write_buffer(&self.args_template, 0, bytemuck::cast_slice(&args));
    queue.write_buffer(&self.late, 0, bytemuck::cast_slice(&late));
    self.initial_args = args;
  }

  /// Every batch's draw arguments as this frame's culls start them.
  pub fn get_initial_args(&self) -> &[u32] {
    &self.initial_args
  }

  /// Entries the visible list holds, and the candidates as many.
  pub fn get_list_capacity(&self) -> u32 {
    self.list_capacity
  }

  /// The bounding sphere of every place that sways, and how far it reaches from its foot, weighted by its rigidity.
  pub fn list_swaying(&self) -> &[(Vec4, f32)] {
    &self.swaying
  }

  /// What the wind's amplitude is multiplied by to give the farthest any tree leans, in metres.
  pub fn get_sway_reach(&self) -> f32 {
    self.sway_reach
  }

  /// The spawned objects with composited surfaces, which a view draws back to front itself.
  pub fn list_sorted_places(&self) -> impl Iterator<Item = StaticSortedPlace<'_>> {
    self.objects.as_slice().iter().filter_map(|object| {
      let model: &StaticModelProxy = self.models.get(object.model)?;

      (!model.sorted.is_empty()).then(|| StaticSortedPlace {
        sphere: object.sphere,
        place: object.place.get_offset(),
        clusters: &model.sorted,
      })
    })
  }

  fn get_object(&self, object: u32) -> Option<&StaticObjectProxy> {
    self.objects.get(*self.object_handles.get(&object)?)
  }

  fn get_object_at(&self, place: u32) -> Option<&StaticObjectProxy> {
    self.objects.get(*self.place_objects.get(&place)?)
  }

  fn find_sector(&self, sector: u32) -> Option<&StaticSectorProxy> {
    self.sectors.as_slice().iter().find(|it| it.sector == sector)
  }

  /// Every span buffer, in the order their bytes and generations are summed.
  fn list_spans(&self) -> [&SpanBuffer; 10] {
    [
      &self.words[0],
      &self.words[1],
      &self.words[2],
      &self.indices,
      &self.clusters,
      &self.slots,
      &self.places,
      &self.impostors,
      &self.skins,
      &self.bones,
    ]
  }

  fn list_spans_mut(&mut self) -> [&mut SpanBuffer; 10] {
    let [baked, tree, model] = &mut self.words;

    [
      baked,
      tree,
      model,
      &mut self.indices,
      &mut self.clusters,
      &mut self.slots,
      &mut self.places,
      &mut self.impostors,
      &mut self.skins,
      &mut self.bones,
    ]
  }

  fn add_capacities(&mut self, capacities: &[u32; StaticBatch::COUNT]) {
    for (total, added) in self.capacities.iter_mut().zip(capacities) {
      *total += added;
    }
  }

  fn remove_capacities(&mut self, capacities: &[u32; StaticBatch::COUNT]) {
    for (total, removed) in self.capacities.iter_mut().zip(capacities) {
      *total -= removed;
    }
  }

  /// Takes a range of each buffer for what a writer gathered, none where it gathered nothing; on an error, what was
  /// taken is given back.
  fn allocate_spans(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    writer: &StaticSceneWriter,
  ) -> XrfResult<StaticSpans> {
    let mut spans: StaticSpans = StaticSpans::default();
    let counts: [usize; 6] = [
      writer.indices.len(),
      writer.clusters.len(),
      writer.slots.len(),
      writer.places.len(),
      writer.impostors.len(),
      writer.skins.len() / 2,
    ];
    let mut result: XrfResult<()> = Ok(());

    for (layout, words) in writer.words.iter().enumerate() {
      if result.is_ok() {
        result = allocate_span(
          &mut self.words[layout],
          (device, queue),
          words.len() / StaticLayout::STRIDE as usize,
        )
        .map(|span| spans.words[layout] = span);
      }
    }

    let buffers: [(&mut SpanBuffer, &mut Option<Span>); 6] = [
      (&mut self.indices, &mut spans.indices),
      (&mut self.clusters, &mut spans.clusters),
      (&mut self.slots, &mut spans.slots),
      (&mut self.places, &mut spans.places),
      (&mut self.impostors, &mut spans.impostors),
      (&mut self.skins, &mut spans.skins),
    ];

    for ((buffer, held), count) in buffers.into_iter().zip(counts) {
      if result.is_ok() {
        result = allocate_span(buffer, (device, queue), count).map(|span| *held = span);
      }
    }

    match result {
      Ok(()) => Ok(spans),
      Err(error) => {
        self.release_spans(spans);

        Err(error)
      }
    }
  }

  /// Queues a writer's records for the ranges taken for them.
  fn write_spans(&mut self, spans: &StaticSpans, writer: &StaticSceneWriter) {
    for (layout, words) in writer.words.iter().enumerate() {
      if let Some(span) = &spans.words[layout] {
        self.words[layout].write(span, 0, bytemuck::cast_slice(words));
      }
    }

    if let Some(span) = &spans.indices {
      self.indices.write(span, 0, bytemuck::cast_slice(&writer.indices));
    }

    if let Some(span) = &spans.clusters {
      self.clusters.write(span, 0, bytemuck::cast_slice(&writer.clusters));
      self
        .clusters
        .write_lane(span, Self::CLUSTER_SPHERES, 0, bytemuck::cast_slice(&writer.spheres));
    }

    if let Some(span) = &spans.slots {
      self.slots.write(span, 0, bytemuck::cast_slice(&writer.slots));
    }

    if let Some(span) = &spans.places {
      self.places.write(span, 0, bytemuck::cast_slice(&writer.places));
    }

    if let Some(span) = &spans.impostors {
      self.impostors.write(span, 0, bytemuck::cast_slice(&writer.impostors));
      self
        .impostors
        .write_lane(span, Self::IMPOSTOR_CORNERS, 0, bytemuck::cast_slice(&writer.corners));
    }

    if let Some(span) = &spans.skins {
      self.skins.write(span, 0, bytemuck::cast_slice(&writer.skins));
    }
  }

  /// Gives an item's ranges back, the clusters and impostors cleared first, since the cull walks them whole.
  fn release_spans(&mut self, spans: StaticSpans) {
    let StaticSpans {
      words,
      indices,
      clusters,
      slots,
      places,
      impostors,
      skins,
    } = spans;

    for (layout, span) in words.into_iter().enumerate() {
      if let Some(span) = span {
        self.words[layout].free(span);
      }
    }

    if let Some(span) = clusters {
      self.clusters.clear(&span);
      self.clusters.free(span);
    }

    if let Some(span) = impostors {
      self.impostors.clear(&span);
      self.impostors.free(span);
    }

    for (buffer, span) in [
      (&mut self.indices, indices),
      (&mut self.slots, slots),
      (&mut self.places, places),
      (&mut self.skins, skins),
    ] {
      if let Some(span) = span {
        buffer.free(span);
      }
    }
  }

  /// Appends the surfaces a writer asked for first, which every item shares from then on.
  fn put_surfaces(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    writer: &StaticSceneWriter,
  ) {
    self
      .surfaces
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.surfaces));
  }

  /// Notes the slots a surface samples: its textures, and apart from them its environment cube's.
  fn note_slots(&mut self, surface: &StaticSurface) {
    for (index, slot) in surface.textures.iter().enumerate() {
      if index == StaticSurface::ENVIRONMENT {
        if *slot != 0 {
          self.environment_slots.insert(*slot);
        }
      } else if *slot != MISSING_SLOT {
        self.texture_slots.insert(*slot);
      }
    }

    // A terrain's details, bumps and mask are sampled as surely as its base.
    self
      .texture_slots
      .extend(surface.terrain.iter_slots().filter(|slot| *slot != MISSING_SLOT));
  }

  fn resolve_surface(
    &mut self,
    writer: &mut StaticSceneWriter,
    (key, surface, descriptor): (StaticSurfaceKey, &SectorSurface, Option<&XraySurfaceDescriptor>),
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) -> Option<(u32, StaticClass)> {
    if let Some(known) = self.surface_rows.get(&key) {
      return *known;
    }

    let is_model: bool = matches!(key, StaticSurfaceKey::Model { .. });
    let built = build_static_surface(surface, descriptor, textures, source);
    let resolved: Option<(u32, StaticClass)> = built.map(|(mut row, class)| {
      if is_model {
        row.flags |= StaticSurface::IS_MODEL;
      }

      self.note_slots(&row);
      writer.surfaces.push(row);
      self.surface_count += 1;

      (self.surface_count - 1, class)
    });

    self.surface_rows.insert(key, resolved);

    resolved
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

    let Some((surface, class)) =
      self.resolve_surface(writer, level_surface(&group.surface, descriptors), textures, source)
    else {
      return;
    };
    let layout: StaticLayout = if geometry.uv_components >= 4 {
      StaticLayout::Tree
    } else {
      StaticLayout::Baked
    };
    let batch: StaticBatch = StaticBatch { layout, class };
    let base: StaticGeometryBase = put_geometry(writer, layout, geometry, buffer);
    let (bands, windows): (Vec<(u32, u32)>, u32) = match &group.progressive {
      Some(progressive) if !progressive.bands.is_empty() => (
        progressive.bands.iter().map(|band| (band.start, band.count)).collect(),
        progressive.windows.max(1),
      ),
      _ => (vec![(0, geometry.index_count)], 1),
    };
    let info: StaticSlotInfo = StaticSlotInfo {
      sector,
      shader_id: group.surface.shader_id,
      mesh: Some(mesh),
      first_place: writer.places.len() as u32,
    };
    let band_slots: Vec<(u32, u32)> = bands
      .iter()
      .map(|range| {
        writer.put_slot(
          &base,
          cut_clusters(geometry, buffer, range.0, range.1),
          (StaticSlot::LISTED, batch, surface),
          info,
        )
      })
      .collect();
    let local: Vec4 =
      bound_spheres(&writer.spheres[writer.spheres.len() - band_slots.iter().map(|it| it.1 as usize).sum::<usize>()..]);
    let transforms: Vec<f32> = read_pods(buffer, &group.transforms);
    let hemi: Vec<f32> = read_pods(buffer, &group.hemi);
    // Each place's impostor, in the sector's own numbering; a tree no impostor stands for has none.
    let lods: Vec<i32> = group
      .impostors
      .as_ref()
      .map_or_else(Vec::new, |it| read_pods(buffer, it));
    let impostors: u32 = writer.impostors.len() as u32;
    let instances: usize = (group.instance_count as usize).min(transforms.len() / 16);
    let sway_reach: f32 = if layout == StaticLayout::Tree {
      measure_sway_reach(geometry, buffer)
    } else {
      0.0
    };

    for instance in 0..instances {
      let matrix: Mat4 = Mat4::from_cols_slice(&transforms[instance * 16..][..16]);
      let scale: f32 = to_scale(&matrix);
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
        writer.capacities[batch.get_index() as usize] += clusters;
      }
    }
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
      let row: u32 = match self.impostor_rows.get(&group.surface.shader_id) {
        Some(row) => *row,
        None => {
          let surface: StaticSurface = build_impostor_surface(&group.surface, textures, source);

          self.note_slots(&surface);
          writer.surfaces.push(surface);
          self.surface_count += 1;
          self
            .impostor_rows
            .insert(group.surface.shader_id, self.surface_count - 1);

          self.surface_count - 1
        }
      };

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
}

/// A range of `count` elements of a buffer, none for no elements.
fn allocate_span(
  buffer: &mut SpanBuffer,
  (device, queue): (&wgpu::Device, &wgpu::Queue),
  count: usize,
) -> XrfResult<Option<Span>> {
  if count == 0 {
    return Ok(None);
  }

  buffer.allocate(device, queue, count as u32).map(Some)
}

/// How full a span buffer is: its elements held, and its capacity.
fn to_pool_use(buffer: &SpanBuffer) -> RenderPoolUse {
  RenderPoolUse {
    used: buffer.get_capacity() - buffer.get_free(),
    capacity: buffer.get_capacity(),
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
    &pack_vertex_words(layout, geometry, buffer),
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
    .step_by((CLUSTER_TRIANGLES * 3) as usize)
    .map(|first| {
      let triangles: u32 = ((end - first) / 3).min(CLUSTER_TRIANGLES);
      let points: Vec<Vec3> = indices
        .iter()
        .skip(first as usize)
        .take((triangles * 3) as usize)
        .filter_map(|index| positions.get(*index as usize).copied().map(Vec3::from_array))
        .collect();

      (first, triangles, bound_points(&points))
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
