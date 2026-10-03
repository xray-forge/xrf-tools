use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use glam::{Mat4, Vec3, Vec4};
use xrf_material::XraySurfaceDescriptor;
use xrf_visual::{SectorGeometry, SectorImpostors, SectorInstanceGroup, SectorPackage, SectorSurface, VisualClusters};

use crate::contract::render_pool_use::RenderPoolUse;
use crate::contract::render_static_report::RenderStaticReport;
use crate::host::render_asset_source::RenderAssetSource;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::section_bytes::read_pods;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_cluster::StaticCluster;
use crate::scene::static_scene::static_impostor::StaticImpostor;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_model::StaticModel;
use crate::scene::static_scene::static_model_place::StaticModelPlace;
use crate::scene::static_scene::static_place::StaticPlace;
use crate::scene::static_scene::static_region::StaticRegion;
use crate::scene::static_scene::static_row::StaticRow;
use crate::scene::static_scene::static_sector::StaticSector;
use crate::scene::static_scene::static_slot::StaticSlot;
use crate::scene::static_scene::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::static_sorted_place::StaticSortedPlace;
use crate::scene::static_scene::static_surface::StaticSurface;
use crate::scene::static_scene::static_surface_build::{build_impostor_surface, build_static_surface};
use crate::scene::static_scene::static_surface_key::StaticSurfaceKey;
use crate::scene::static_scene::static_vertex_words::{measure_sway_reach, pack_vertex_words};
use crate::scene::texture::texture_cache::{MISSING_SLOT, TextureCache};

/// Clusters a spawned model's row holds at most: the row cull walks a row's clusters on one thread.
const MODEL_ROW_CLUSTERS: usize = 32;

/// Triangles a cluster holds at most.
const CLUSTER_TRIANGLES: u32 = VisualClusters::MAX_TRIANGLES;

/// A level's static geometry on the GPU: arenas of vertices and indices every sector appends to, the clusters the cull
/// tests, the slots, places and rows they are drawn as, and the surfaces they wear.
pub struct StaticScene {
  pub words: [GrowableBuffer; StaticLayout::COUNT],
  pub indices: GrowableBuffer,
  pub clusters: GrowableBuffer,
  pub spheres: GrowableBuffer,
  pub slots: GrowableBuffer,
  pub places: GrowableBuffer,
  pub rows: GrowableBuffer,
  pub surfaces: GrowableBuffer,
  pub lists: GrowableBuffer,
  /// Clusters the early phase set aside, for the late phase to test again.
  pub candidates: GrowableBuffer,
  pub impostors: GrowableBuffer,
  /// Skinned models' links, two words a vertex, and their places' bone matrices, three rows a bone.
  pub skins: GrowableBuffer,
  pub bones: GrowableBuffer,
  /// Two a corner of each impostor: its position and hemisphere term, then its atlas coordinate and sun term.
  pub corners: GrowableBuffer,
  /// What each impostor's level of detail draws this frame, as the cull decided it.
  pub terms: GrowableBuffer,
  /// The impostors drawn this frame.
  pub impostor_list: GrowableBuffer,
  pub regions: wgpu::Buffer,
  pub args: wgpu::Buffer,
  /// How the trees sway this frame, which every draw of the scene reads.
  pub wind: wgpu::Buffer,
  /// Each batch's late draw arguments, then the late phase's dispatch and the candidates' count.
  pub late: wgpu::Buffer,
  /// The late phase's dispatch, copied out of `late` so the dispatch reads no buffer it binds.
  pub late_dispatch: wgpu::Buffer,
  /// Every batch's draw arguments as a cull starts them, copied into a view's own before each cull of it in a frame.
  pub args_template: wgpu::Buffer,
  /// Entries the visible list and the candidates hold this frame.
  list_capacity: u32,
  /// Every batch's draw arguments as a cull starts them, which a shadow's own arguments start from too.
  initial_args: Vec<u32>,
  /// Vertices in each layout's arena, which a geometry's indices are offset by.
  vertex_counts: [u32; StaticLayout::COUNT],
  index_count: u32,
  cluster_count: u32,
  slot_count: u32,
  place_count: u32,
  row_count: u32,
  impostor_count: u32,
  /// Each impostor's sector and shader table entry, which turns a pick of it back into names.
  impostor_infos: Vec<(u32, u16)>,
  /// Each impostor surface's row, by its shader table entry.
  impostor_rows: HashMap<u16, u32>,
  /// Each surface's row and class, once asked for; `None` for one no static draw draws.
  surface_rows: HashMap<StaticSurfaceKey, Option<(u32, StaticClass)>>,
  /// Every texture slot the scene's surfaces sample, in the order they were first asked for.
  pub texture_slots: BTreeSet<u32>,
  surface_count: u32,
  /// Entries of the visible list each batch may need: its clusters, each counted once per place drawing it.
  capacities: [u32; StaticBatch::COUNT],
  pub sectors: Vec<StaticSector>,
  /// Each slot's part, and each cluster's slot, on the CPU: what turns a pick back into names.
  slot_infos: Vec<StaticSlotInfo>,
  cluster_slots: Vec<u32>,
  /// The bounding sphere of every place that sways and its reach, which a light's shadow looks for in its range.
  swaying: Vec<(Vec4, f32)>,
  /// The farthest any swaying vertex reaches from its tree's foot, weighted by its rigidity and scaled by its place.
  sway_reach: f32,
  /// Sectors and models added, which a view drawn with fewer is drawn again for.
  contents: usize,
  /// The spawned object standing at each of the models' places, which a pick of one names.
  place_objects: HashMap<u32, u32>,
  /// Each skinned object's bone matrices: their first row in `bones`, and how many bones.
  skinned: HashMap<u32, (u32, u32)>,
  /// Where each spawned object stands, by its index.
  object_transforms: HashMap<u32, Mat4>,
  /// Each spawned object's bounding sphere in renderer space, by its index, as its model stands in its place.
  object_spheres: HashMap<u32, Vec4>,
  /// The models' places with composited surfaces, which a view draws back to front itself.
  pub sorted_places: Vec<StaticSortedPlace>,
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

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let storage: wgpu::BufferUsages = wgpu::BufferUsages::STORAGE;
    let mut scene: Self = Self {
      words: [
        GrowableBuffer::new(device, "static baked vertices", storage),
        GrowableBuffer::new(device, "static tree vertices", storage),
        GrowableBuffer::new(device, "static model vertices", storage),
      ],
      indices: GrowableBuffer::new(device, "static indices", storage),
      clusters: GrowableBuffer::new(device, "static clusters", storage),
      spheres: GrowableBuffer::new(device, "static spheres", storage),
      slots: GrowableBuffer::new(device, "static slots", storage),
      places: GrowableBuffer::new(device, "static places", storage),
      rows: GrowableBuffer::new(device, "static rows", storage),
      surfaces: GrowableBuffer::new(device, "static surfaces", storage),
      lists: GrowableBuffer::new(device, "static lists", storage),
      candidates: GrowableBuffer::new(device, "static candidates", storage),
      impostors: GrowableBuffer::new(device, "static impostors", storage),
      skins: GrowableBuffer::new(device, "static model skins", storage),
      bones: GrowableBuffer::new(device, "static bones", storage),
      corners: GrowableBuffer::new(device, "static impostor corners", storage),
      terms: GrowableBuffer::new(device, "static impostor terms", storage),
      impostor_list: GrowableBuffer::new(device, "static impostor list", storage),
      regions: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("static regions"),
        size: (StaticBatch::COUNT * size_of::<StaticRegion>()) as u64,
        usage: storage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      wind: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("static wind"),
        size: size_of::<WindUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
      list_capacity: 0,
      initial_args: Vec::new(),
      vertex_counts: [0; StaticLayout::COUNT],
      index_count: 0,
      cluster_count: 0,
      slot_count: 0,
      place_count: 0,
      row_count: 0,
      impostor_count: 0,
      impostor_infos: Vec::new(),
      impostor_rows: HashMap::new(),
      surface_rows: HashMap::new(),
      texture_slots: BTreeSet::new(),
      surface_count: 0,
      capacities: [0; StaticBatch::COUNT],
      sectors: Vec::new(),
      slot_infos: Vec::new(),
      cluster_slots: Vec::new(),
      swaying: Vec::new(),
      sway_reach: 0.0,
      contents: 0,
      place_objects: HashMap::new(),
      skinned: HashMap::new(),
      object_transforms: HashMap::new(),
      object_spheres: HashMap::new(),
      sorted_places: Vec::new(),
    };
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

    // Place zero is where every single draw stands: its geometry is in renderer space already.
    scene
      .places
      .append(device, queue, &mut encoder, bytemuck::bytes_of(&StaticPlace::default()));
    scene.place_count = 1;
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
      slots: self.slots.get_use(size_of::<StaticSlot>()),
      places: self.places.get_use(size_of::<StaticPlace>()),
      rows: self.rows.get_use(size_of::<StaticRow>()),
      lods: self.impostors.get_use(size_of::<StaticImpostor>()),
      clusters: self.clusters.get_use(size_of::<StaticCluster>()),
      surface_list: RenderPoolUse {
        used: 0,
        capacity: self.list_capacity,
      },
      ..Default::default()
    }
  }

  pub fn get_cluster_count(&self) -> u32 {
    self.cluster_count
  }

  pub fn get_row_count(&self) -> u32 {
    self.row_count
  }

  pub fn get_impostor_count(&self) -> u32 {
    self.impostor_count
  }

  /// The sector and shader table entry of a picked impostor.
  pub fn resolve_impostor_pick(&self, impostor: u32) -> Option<(u32, u16)> {
    self.impostor_infos.get(impostor as usize).copied()
  }

  /// What a pick's cluster and place were part of: its slot's part, and the place's instance where it is a mesh's.
  pub fn resolve_pick(&self, cluster: u32, place: u32) -> Option<(StaticSlotInfo, Option<u32>)> {
    let info: StaticSlotInfo = *self
      .slot_infos
      .get(*self.cluster_slots.get(cluster as usize)? as usize)?;

    Some((info, info.mesh.map(|_| place.saturating_sub(info.first_place))))
  }

  /// The spawned object a picked place is, where it is one.
  pub fn resolve_spawn_pick(&self, place: u32) -> Option<u32> {
    self.place_objects.get(&place).copied()
  }

  /// A spawned object's bounding sphere in renderer space, once its model is in the scene.
  pub fn get_object_sphere(&self, object: u32) -> Option<Vec4> {
    self.object_spheres.get(&object).copied()
  }

  /// Where a spawned object stands, once its model is in the scene.
  pub fn get_object_transform(&self, object: u32) -> Option<Mat4> {
    self.object_transforms.get(&object).copied()
  }

  /// How many bones a skinned object's model has, once it is in the scene; none for one rigid or not in it.
  pub fn get_bone_count(&self, object: u32) -> Option<u32> {
    self.skinned.get(&object).map(|(_, bones)| *bones)
  }

  /// Stands a skinned object in a pose: each bone's matrix as three rows, from its bind to where it stands, this frame
  /// and the last, as many as its model has bones each.
  pub fn write_pose(&self, queue: &wgpu::Queue, object: u32, current: &[[Vec4; 3]], previous: &[[Vec4; 3]]) {
    if let Some((base, bones)) = self.skinned.get(&object) {
      let count: usize = *bones as usize;
      let mut rows: Vec<[Vec4; 3]> = vec![[Vec4::X, Vec4::Y, Vec4::Z]; count * 2];

      for (to, from) in rows[..count].iter_mut().zip(current) {
        *to = *from;
      }

      for (to, from) in rows[count..].iter_mut().zip(previous) {
        *to = *from;
      }

      queue.write_buffer(
        self.bones.get_buffer(),
        u64::from(*base) * 16,
        bytemuck::cast_slice(&rows),
      );
    }
  }

  /// Bytes of every pack put into the scene.
  pub fn get_bytes(&self) -> u64 {
    self.sectors.iter().map(|sector| sector.bytes).sum()
  }

  /// A generation of every buffer a bind group holds, which changes whenever one of them was replaced.
  pub fn get_generation(&self) -> u64 {
    [
      &self.words[0],
      &self.words[1],
      &self.words[2],
      &self.indices,
      &self.clusters,
      &self.spheres,
      &self.slots,
      &self.places,
      &self.rows,
      &self.surfaces,
      &self.lists,
      &self.candidates,
      &self.impostors,
      &self.skins,
      &self.bones,
      &self.corners,
      &self.terms,
      &self.impostor_list,
    ]
    .iter()
    .map(|buffer| buffer.get_generation())
    .sum()
  }

  /// Puts one packed sector into the scene: its baked sections as single draws, its tree groups as rows.
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
  ) {
    let mut writer: SceneWriter = SceneWriter::default();
    let first_slot: u32 = self.slot_count;
    let description = &package.description;
    let buffer: &[u8] = &package.buffer;

    if !description.sections.is_empty() && description.geometry.index_count > 0 {
      let base: GeometryBase = self.put_geometry(&mut writer, StaticLayout::Baked, &description.geometry, buffer);

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
        let clusters: u32 = self.put_slot(
          &mut writer,
          &base,
          cut_clusters(&description.geometry, buffer, section.draw.start, section.draw.count),
          StaticSlot::SINGLE,
          batch,
          surface,
          StaticSlotInfo {
            sector: description.sector,
            shader_id: section.surface.shader_id,
            mesh: None,
            first_place: 0,
          },
        );

        self.capacities[batch.get_index() as usize] += clusters;
      }
    }

    let first_impostor: u32 = self.impostor_count;

    if let Some(impostors) = &description.impostors {
      self.put_impostors(&mut writer, description.sector, impostors, buffer, textures, source);
    }

    let impostors: (u32, u32) = (first_impostor, self.impostor_count - first_impostor);

    for (mesh, group) in description.instances.iter().enumerate() {
      self.put_instances(
        &mut writer,
        description.sector,
        mesh as u32,
        group,
        buffer,
        impostors,
        descriptors,
        textures,
        source,
      );
    }

    self.flush(device, queue, encoder, writer);
    self.sectors.push(StaticSector {
      sector: description.sector,
      slots: first_slot..self.slot_count,
      bytes: buffer.len() as u64,
    });
    self.contents += 1;
  }

  /// Puts a spawned model and every object standing as it: its geometry once, a slot a part, a place an object and a
  /// row an object and part, each row shown by its object's group.
  #[allow(clippy::too_many_arguments)]
  pub fn add_model(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
    model: &StaticModel,
    places: &[StaticModelPlace],
  ) {
    let mut writer: SceneWriter = SceneWriter::default();
    let base: GeometryBase = self.put_words(&mut writer, StaticLayout::Model, &model.words, &model.indices);
    let first_place: u32 = self.place_count;
    let mut slots: Vec<(u32, u32, StaticBatch)> = Vec::new();
    // The composited parts' clusters, which each place lists to be drawn sorted.
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
        let slot: u32 = self.slot_count;
        let first_cluster: u32 = self.cluster_count;
        let count: u32 = self.put_slot(
          &mut writer,
          &base,
          clusters.to_vec(),
          StaticSlot::LISTED,
          batch,
          surface,
          StaticSlotInfo {
            sector: StaticSlotInfo::NO_SECTOR,
            shader_id: part.surface.shader_id,
            mesh: None,
            first_place,
          },
        );

        if class == StaticClass::Composited {
          sorted.push((first_cluster, count));
        }

        slots.push((slot, count, batch));
      }
    }

    // The links once a model, by vertex; each place reserves its own bone matrices, standing in the bind pose.
    let skin_base: Option<(u32, u32)> = model.skin.as_ref().map(|skin| {
      let start: u64 = self
        .skins
        .append(device, queue, encoder, bytemuck::cast_slice(&skin.links));

      ((start / 8) as u32, skin.bones)
    });

    for place in places {
      let matrix: Mat4 = place.transform;
      let scale: f32 = [matrix.x_axis, matrix.y_axis, matrix.z_axis]
        .iter()
        .map(|axis| axis.truncate().length())
        .fold(0.0, f32::max);
      let index: u32 = self.place_count;
      let skin: [u32; 4] = skin_base.map_or([0; 4], |(links, bones)| {
        let bones: u32 = bones.max(1);
        // This frame's matrices and the last's, both the bind pose until a pose is written.
        let bind: Vec<[Vec4; 3]> = vec![[Vec4::X, Vec4::Y, Vec4::Z]; bones as usize * 2];
        let rows: u32 = (self.bones.append(device, queue, encoder, bytemuck::cast_slice(&bind)) / 16) as u32;

        self.skinned.insert(place.object, (rows, bones));

        [rows, links, base.vertex_start, bones]
      });

      self.place_objects.insert(index, place.object);
      self.object_transforms.insert(place.object, matrix);
      self.object_spheres.insert(
        place.object,
        matrix
          .transform_point3(model.sphere.truncate())
          .extend(model.sphere.w * scale),
      );

      if !sorted.is_empty() {
        self.sorted_places.push(StaticSortedPlace {
          sphere: matrix
            .transform_point3(model.sphere.truncate())
            .extend(model.sphere.w * scale),
          place: index,
          group: place.group,
          clusters: sorted.clone(),
        });
      }

      writer.places.push(StaticPlace {
        columns: [matrix.x_axis, matrix.y_axis, matrix.z_axis, matrix.w_axis],
        info: Vec4::new(1.0, 0.0, -1.0, scale),
        cube: place.lighting.map_or([0; 4], pack_cube),
        skin,
      });
      self.place_count += 1;

      for (slot, clusters, batch) in &slots {
        writer.rows.push(StaticRow {
          sphere: matrix
            .transform_point3(model.sphere.truncate())
            .extend(model.sphere.w * scale),
          place: index,
          slot: *slot,
          lod: StaticRow::NO_LOD,
          band: StaticRow::pack_band(0, 1, 1) | StaticRow::pack_group(place.group),
        });
        self.row_count += 1;
        self.capacities[batch.get_index() as usize] += clusters;
      }
    }

    self.flush(device, queue, encoder, writer);
    self.contents += 1;
  }

  /// Sectors and models added so far, which a view drawn with fewer is drawn again for.
  pub fn get_contents(&self) -> usize {
    self.contents
  }

  /// Rewrites each batch's run of the visible list, and its draw arguments empty, for a frame's cull to fill.
  pub fn reset_draws(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, encoder: &mut wgpu::CommandEncoder) {
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
    self
      .terms
      .reserve(device, encoder, (self.impostor_count.max(1) as u64) * 16);
    self
      .impostor_list
      .reserve(device, encoder, (self.impostor_count.max(1) as u64) * 4);
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

  fn put_geometry(
    &mut self,
    writer: &mut SceneWriter,
    layout: StaticLayout,
    geometry: &SectorGeometry,
    buffer: &[u8],
  ) -> GeometryBase {
    self.put_words(
      writer,
      layout,
      &pack_vertex_words(layout, geometry, buffer),
      &read_pods::<u32>(buffer, &geometry.indices),
    )
  }

  /// Appends a geometry's vertex words to its layout's arena and its indices to the shared one.
  fn put_words(
    &mut self,
    writer: &mut SceneWriter,
    layout: StaticLayout,
    words: &[u32],
    indices: &[u32],
  ) -> GeometryBase {
    let base: GeometryBase = GeometryBase {
      layout,
      vertex_start: self.vertex_counts[layout.get_index()],
      index_start: self.index_count,
    };

    self.vertex_counts[layout.get_index()] += (words.len() / StaticLayout::STRIDE as usize) as u32;
    self.index_count += indices.len() as u32;
    writer.words[layout.get_index()].extend_from_slice(words);
    writer.indices.extend_from_slice(indices);

    base
  }

  fn resolve_surface(
    &mut self,
    writer: &mut SceneWriter,
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

      self
        .texture_slots
        .extend(row.textures.iter().filter(|slot| **slot != MISSING_SLOT));
      writer.surfaces.push(row);
      self.surface_count += 1;

      (self.surface_count - 1, class)
    });

    self.surface_rows.insert(key, resolved);

    resolved
  }

  /// Puts the clusters covering one index range as one slot; answers how many clusters it took.
  #[allow(clippy::too_many_arguments)]
  fn put_slot(
    &mut self,
    writer: &mut SceneWriter,
    base: &GeometryBase,
    clusters: Vec<(u32, u32, Vec4)>,
    kind: u32,
    batch: StaticBatch,
    surface: u32,
    info: StaticSlotInfo,
  ) -> u32 {
    let slot: u32 = self.slot_count;
    let first_cluster: u32 = self.cluster_count;

    for (first_index, triangles, sphere) in clusters {
      writer.clusters.push(StaticCluster {
        first_index: base.index_start + first_index,
        triangles,
        vertex_start: base.vertex_start,
        slot,
      });
      writer.spheres.push(sphere);
      self.cluster_slots.push(slot);
      self.cluster_count += 1;
    }

    writer.slots.push(StaticSlot {
      first_cluster,
      cluster_count: self.cluster_count - first_cluster,
      place: 0,
      kind,
      batch: batch.get_index(),
      surface,
      pad: [0; 2],
    });
    self.slot_infos.push(info);
    self.slot_count += 1;

    self.cluster_count - first_cluster
  }

  /// Puts an instanced group: its geometry once, a slot per progressive band, a place per instance and a row per
  /// instance and band.
  #[allow(clippy::too_many_arguments)]
  fn put_instances(
    &mut self,
    writer: &mut SceneWriter,
    sector: u32,
    mesh: u32,
    group: &SectorInstanceGroup,
    buffer: &[u8],
    (first_impostor, impostor_count): (u32, u32),
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
    let base: GeometryBase = self.put_geometry(writer, layout, geometry, buffer);
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
      first_place: self.place_count,
    };
    let band_slots: Vec<(u32, u32)> = bands
      .iter()
      .map(|range| {
        let slot: u32 = self.slot_count;
        let clusters: u32 = self.put_slot(
          writer,
          &base,
          cut_clusters(geometry, buffer, range.0, range.1),
          StaticSlot::LISTED,
          batch,
          surface,
          info,
        );

        (slot, clusters)
      })
      .collect();
    let local: Vec4 =
      bound_spheres(&writer.spheres[writer.spheres.len() - band_slots.iter().map(|it| it.1 as usize).sum::<usize>()..]);
    let transforms: Vec<f32> = read_pods(buffer, &group.transforms);
    let hemi: Vec<f32> = read_pods(buffer, &group.hemi);
    // Each place's impostor, the sector's own numbering moved to the scene's; a tree no impostor stands for has none.
    let lods: Vec<i32> = group
      .impostors
      .as_ref()
      .map_or_else(Vec::new, |it| read_pods(buffer, it));
    let instances: usize = (group.instance_count as usize).min(transforms.len() / 16);
    let sway_reach: f32 = if layout == StaticLayout::Tree {
      measure_sway_reach(geometry, buffer)
    } else {
      0.0
    };

    for instance in 0..instances {
      let matrix: Mat4 = Mat4::from_cols_slice(&transforms[instance * 16..][..16]);
      let scale: f32 = [matrix.x_axis, matrix.y_axis, matrix.z_axis]
        .iter()
        .map(|axis| axis.truncate().length())
        .fold(0.0, f32::max);
      let (hemi_scale, hemi_bias): (f32, f32) = (
        hemi.get(instance * 2).copied().unwrap_or(1.0),
        hemi.get(instance * 2 + 1).copied().unwrap_or(0.0),
      );
      let place: u32 = self.place_count;
      let center: Vec3 = matrix.transform_point3(local.truncate());

      if sway_reach > 0.0 {
        self.swaying.push((center.extend(local.w * scale), sway_reach * scale));
        self.sway_reach = self.sway_reach.max(sway_reach * scale);
      }
      let lod: u32 = lods
        .get(instance)
        .and_then(|it| u32::try_from(*it).ok())
        .filter(|it| *it < impostor_count)
        .map_or(StaticRow::NO_LOD, |it| first_impostor + it);

      writer.places.push(StaticPlace {
        columns: [matrix.x_axis, matrix.y_axis, matrix.z_axis, matrix.w_axis],
        info: Vec4::new(
          hemi_scale,
          hemi_bias,
          if lod == StaticRow::NO_LOD { -1.0 } else { lod as f32 },
          scale,
        ),
        cube: [0; 4],
        skin: [0; 4],
      });
      self.place_count += 1;

      for (band, (slot, clusters)) in band_slots.iter().enumerate() {
        writer.rows.push(StaticRow {
          sphere: center.extend(local.w * scale),
          place,
          slot: *slot,
          lod,
          band: StaticRow::pack_band(band as u32, band_slots.len() as u32, windows),
        });
        self.row_count += 1;
        self.capacities[batch.get_index() as usize] += clusters;
      }
    }
  }

  /// Puts a sector's impostors in its own order, each with its run's surface; one no run dresses is never drawn.
  fn put_impostors(
    &mut self,
    writer: &mut SceneWriter,
    sector: u32,
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

          self
            .texture_slots
            .extend(surface.textures.iter().filter(|slot| **slot != MISSING_SLOT));
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
        pad: [0; 2],
      });

      for corner in &corners[index * StaticImpostor::CORNERS..][..StaticImpostor::CORNERS] {
        // Position, atlas coordinate, then the hemisphere and sun terms.
        writer
          .corners
          .push(Vec4::new(corner[0], corner[1], corner[2], corner[5]));
        writer.corners.push(Vec4::new(corner[3], corner[4], corner[6], 0.0));
      }

      self.impostor_infos.push((sector, dressing.map_or(0, |it| it.1)));
    }

    self.impostor_count += count as u32;
  }

  fn flush(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    writer: SceneWriter,
  ) {
    for (index, words) in writer.words.iter().enumerate() {
      self.words[index].append(device, queue, encoder, bytemuck::cast_slice(words));
    }

    self
      .indices
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.indices));
    self
      .clusters
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.clusters));
    self
      .spheres
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.spheres));
    self
      .slots
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.slots));
    self
      .places
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.places));
    self
      .rows
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.rows));
    self
      .surfaces
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.surfaces));
    self
      .impostors
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.impostors));
    self
      .corners
      .append(device, queue, encoder, bytemuck::cast_slice(&writer.corners));
  }
}

/// Where one geometry's vertices and indices start in the arenas.
struct GeometryBase {
  layout: StaticLayout,
  vertex_start: u32,
  index_start: u32,
}

/// A spawned object's lighting as a place holds it: each cube face a byte, `+x +y +z -x` in `x`, `-y -z` in `y`, the
/// sky share's bits in `z`, and `w` marking it.
fn pack_cube((faces, sky): ([f32; 6], f32)) -> [u32; 4] {
  let bytes: [u32; 6] = faces.map(|face| (face.clamp(0.0, 1.0) * 255.0).round() as u32);

  [
    bytes[0] | (bytes[1] << 8) | (bytes[2] << 16) | (bytes[3] << 24),
    bytes[4] | (bytes[5] << 8),
    sky.to_bits(),
    1,
  ]
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

/// What one sector adds, gathered before it is written in one append a buffer.
#[derive(Default)]
struct SceneWriter {
  words: [Vec<u32>; StaticLayout::COUNT],
  indices: Vec<u32>,
  clusters: Vec<StaticCluster>,
  spheres: Vec<Vec4>,
  slots: Vec<StaticSlot>,
  places: Vec<StaticPlace>,
  rows: Vec<StaticRow>,
  surfaces: Vec<StaticSurface>,
  impostors: Vec<StaticImpostor>,
  corners: Vec<Vec4>,
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
