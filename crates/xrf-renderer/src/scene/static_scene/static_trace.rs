use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use xrf_renderer_core::{AccelerationStructureBinding, ProxyHandle};

use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::static_trace_group::StaticTraceGroup;
use crate::scene::static_scene::static_trace_instance::StaticTraceInstance;
use crate::scene::static_scene::static_trace_record::StaticTraceRecord;
use crate::scene::static_scene::static_trace_swaying::StaticTraceSwaying;

/// The level's static geometry as rays are traced against it by hardware ray query: bottom-level acceleration
/// structures read in place from the scene's vertex and index arenas, a sector's deferred baked sections in one, each
/// of its instanced meshes and each rigid spawned model in one of its own; and one top-level structure, an instance for
/// each sector's baked geometry, for each place an instanced mesh stands at and for each object a model stands as. Each
/// instance names an entry of the instance table, its first record and its place, so a hit finds its geometry, its
/// triangle's corners, its surface and its hemisphere (`TraceParameters`). Built again as sectors come and go, whole
/// when an arena it reads grows.
pub struct StaticTrace {
  tlas: wgpu::Tlas,
  /// The key the top-level structure is bound by, a new one each time it is made.
  key: u64,
  instances: wgpu::Buffer,
  records: wgpu::Buffer,
  /// What a trace binding with nothing to hold is given: a few records of nothing.
  empty: wgpu::Buffer,
  /// One triangle no ray meets, the top level's instance while it holds nothing else: a top level of no instances is a
  /// build some drivers fail (WARP divides by its instance count).
  placeholder: wgpu::Blas,
  /// Each sector's structures, by its index in the level.
  sectors: HashMap<u32, SectorStructures>,
  /// Each traced model's structure, of those objects stand as.
  models: HashMap<ProxyHandle<StaticModelProxy>, wgpu::Blas>,
  /// The scene's contents and the generations of the arenas read, as last built over.
  built: Option<(usize, [u64; StaticLayout::COUNT + 1])>,
  /// When the top level was last built, which a change that only adds waits on.
  built_at: Option<Instant>,
  /// The sectors and the objects' places the top level was last built over.
  built_over: (HashSet<u32>, HashSet<u32>),
  /// Every top-level instance as last built, its structure and transform, by its index.
  built_instances: Vec<(Structure, [f32; 12])>,
  /// The instances of trees' geometry, which the wind sways.
  swaying: Vec<StaticTraceSwaying>,
  /// Builds made, which a view's own top level is filled again after.
  version: u64,
}

/// One sector's bottom-level structures, and the first index it was built over, which a sector moved in its arena
/// changes: its baked sections', where it has any, and each instanced mesh's.
struct SectorStructures {
  baked: Option<wgpu::Blas>,
  groups: Vec<wgpu::Blas>,
  first_index: u32,
}

/// What a bottom-level structure is built over.
#[derive(Clone, Copy, Debug)]
enum Structure {
  /// A sector's baked sections, by the sector's index.
  Baked(u32),
  /// One of a sector's instanced meshes, by the sector's index and the mesh's.
  Group(u32, usize),
  Model(ProxyHandle<StaticModelProxy>),
}

/// A bottom-level structure waiting to be built over its geometries.
struct PendingBuild<'s> {
  structure: Structure,
  geometries: &'s [StaticTraceGeometry],
  sizes: Vec<wgpu::BlasTriangleGeometrySizeDescriptor>,
}

impl StaticTrace {
  /// The least time between two builds over contents that only grew, so a level streaming in builds a few times a
  /// second rather than every frame; a change that took anything out, or moved the arenas, builds at once.
  const REBUILD_EVERY: Duration = Duration::from_millis(250);

  /// Instances the top-level structure holds at first; it is made again twice as large when outgrown.
  const INSTANCES: u32 = 64;

  /// Records and instance table entries their buffers hold at first.
  const RECORDS: u64 = 1024;

  /// An instance's transform where the geometry stands in renderer space already.
  pub const IDENTITY: [f32; 12] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];
  /// Every scene instance's mask (`SCENE_MASK_SCENE`), which a ray asking for the scene alone meets.
  pub const MASK: u8 = 0x01;

  /// The place every sector's baked geometry stands at: the scene's origin, whose hemisphere is the vertices' own.
  const ORIGIN_PLACE: u32 = 0;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let mut tlas: wgpu::Tlas = Self::create_tlas(device, Self::INSTANCES);
    let size: wgpu::BlasTriangleGeometrySizeDescriptor = Self::get_placeholder_size();
    let placeholder: wgpu::Blas = device.create_blas(
      &wgpu::CreateBlasDescriptor {
        label: Some("static trace placeholder"),
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
      },
      wgpu::BlasGeometrySizeDescriptors::Triangles {
        descriptors: vec![size.clone()],
      },
    );
    let corners: wgpu::Buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some("static trace placeholder"),
      contents: bytemuck::cast_slice(&[[0.0_f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]]),
      usage: wgpu::BufferUsages::BLAS_INPUT,
    });
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("static trace"),
    });

    // Built over the placeholder alone, so it is bound built before any sector is in.
    tlas[0] = Some(Self::create_placeholder_instance(&placeholder));
    encoder.build_acceleration_structures(
      std::iter::once(&wgpu::BlasBuildEntry {
        blas: &placeholder,
        geometry: wgpu::BlasGeometries::TriangleGeometries(vec![wgpu::BlasTriangleGeometry {
          size: &size,
          vertex_buffer: &corners,
          first_vertex: 0,
          vertex_stride: 12,
          index_buffer: None,
          first_index: None,
          transform_buffer: None,
          transform_buffer_offset: None,
        }]),
      }),
      std::iter::once(&tlas),
    );
    queue.submit([encoder.finish()]);

    Self {
      tlas,
      key: AccelerationStructureBinding::next_key(),
      instances: Self::create_table::<StaticTraceInstance>(device, "static trace instances", Self::RECORDS),
      records: Self::create_table::<StaticTraceRecord>(device, "static trace records", Self::RECORDS),
      empty: Self::create_table::<[u32; 16]>(device, "static trace empty", 1),
      placeholder,
      sectors: HashMap::new(),
      models: HashMap::new(),
      built: None,
      built_at: None,
      built_over: (HashSet::new(), HashSet::new()),
      built_instances: Vec::new(),
      swaying: Vec::new(),
      version: 0,
    }
  }

  /// The top-level structure, as a pass binds it.
  pub fn get_structure(&self) -> AccelerationStructureBinding<'_> {
    AccelerationStructureBinding::new(&self.tlas, self.key)
  }

  /// Each instance's first record and place, by its custom index.
  pub fn get_instances(&self) -> &wgpu::Buffer {
    &self.instances
  }

  /// What a trace binding with nothing to hold is given, such as the grass of a level without any.
  pub fn get_empty(&self) -> &wgpu::Buffer {
    &self.empty
  }

  /// Each geometry's record, an instance's in a run its table entry names the first of.
  pub fn get_records(&self) -> &wgpu::Buffer {
    &self.records
  }

  /// The instances of trees' geometry, which the wind sways, as last built.
  pub fn get_swaying(&self) -> &[StaticTraceSwaying] {
    &self.swaying
  }

  /// Top-level instances as last built.
  pub fn get_instance_count(&self) -> usize {
    self.built_instances.len()
  }

  /// Fills a view's own top level with every instance as last built, those `swayed` names standing as the structure
  /// given beside each, its geometry in renderer space already.
  pub fn fill_view_tlas(&self, tlas: &mut wgpu::Tlas, swayed: &HashMap<u32, &wgpu::Blas>) {
    for index in 0..tlas.get().len() {
      tlas[index] = self
        .built_instances
        .get(index)
        .map(|(structure, transform)| match swayed.get(&(index as u32)) {
          Some(blas) => wgpu::TlasInstance::new(blas, Self::IDENTITY, index as u32, Self::MASK),
          None => wgpu::TlasInstance::new(
            Self::find_blas(&self.sectors, &self.models, *structure),
            *transform,
            index as u32,
            Self::MASK,
          ),
        });
    }
  }

  /// Builds what changed in the scene since the last call into `encoder`: the structures of each sector come in or
  /// moved and each model come in, every one where an arena grew, and the top level over them all.
  pub fn update(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    statics: &StaticScene,
  ) {
    let generations: [u64; StaticLayout::COUNT + 1] = [
      statics.geometry.words[0].get_generation(),
      statics.geometry.words[1].get_generation(),
      statics.geometry.words[2].get_generation(),
      statics.geometry.indices.get_generation(),
    ];
    let built: (usize, [u64; StaticLayout::COUNT + 1]) = (statics.get_contents(), generations);

    let has_moved: bool = self.built.is_some_and(|(_, held)| held != generations);

    if has_moved {
      self.sectors.clear();
      self.models.clear();
    }

    let is_due: bool =
      has_moved || !self.is_only_added(statics) || self.built_at.is_none_or(|at| at.elapsed() >= Self::REBUILD_EVERY);

    if self.built != Some(built) && is_due {
      self.built = Some(built);
      self.built_at = Some(Instant::now());
      self.build(device, queue, encoder, statics);
    }
  }

  /// Whether every sector and object place built over is still in the scene, so what changed only added to it.
  fn is_only_added(&self, statics: &StaticScene) -> bool {
    let (sectors, places) = &self.built_over;
    let present_sectors: HashSet<u32> = statics.list_trace_sectors().map(|(sector, ..)| sector).collect();
    let present_places: HashSet<u32> = statics.list_trace_objects().map(|(_, place, _)| place).collect();

    sectors.is_subset(&present_sectors) && places.is_subset(&present_places)
  }

  fn build(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    statics: &StaticScene,
  ) {
    let sectors: Vec<(u32, &[StaticTraceGeometry], &[StaticTraceGroup])> = statics
      .list_trace_sectors()
      .filter(|(_, sections, groups)| !sections.is_empty() || !groups.is_empty())
      .collect();
    let objects: Vec<(ProxyHandle<StaticModelProxy>, u32, &Mat4)> = statics.list_trace_objects().collect();
    let standing: HashSet<ProxyHandle<StaticModelProxy>> = objects.iter().map(|(model, ..)| *model).collect();
    // Only the models an object stands as: one whose objects all left the scene takes no memory.
    let models: Vec<(ProxyHandle<StaticModelProxy>, &[StaticTraceGeometry])> = statics
      .list_trace_models()
      .filter(|(model, _)| standing.contains(model))
      .collect();

    self
      .sectors
      .retain(|sector, _| sectors.iter().any(|(present, ..)| present == sector));
    self.models.retain(|model, _| standing.contains(model));

    let pending: Vec<PendingBuild<'_>> = self.create_structures(device, &sectors, &models);
    let (table, records, instances, swaying) = Self::list_instances(&sectors, &models, &objects);

    self.write_tables(device, queue, &table, &records);

    if instances.len() > self.tlas.get().len() {
      self.tlas = Self::create_tlas(device, (instances.len() as u32).next_power_of_two());
      self.key = AccelerationStructureBinding::next_key();
    }

    let entries: Vec<wgpu::BlasBuildEntry<'_>> = pending
      .iter()
      .map(|build| wgpu::BlasBuildEntry {
        blas: Self::find_blas(&self.sectors, &self.models, build.structure),
        geometry: wgpu::BlasGeometries::TriangleGeometries(
          build
            .geometries
            .iter()
            .zip(&build.sizes)
            .map(|(geometry, size)| wgpu::BlasTriangleGeometry {
              size,
              // Each vertex's position is its first three words, so the arena is read in place.
              vertex_buffer: statics.geometry.words[geometry.layout.get_index()].get_buffer(),
              first_vertex: geometry.vertex_start,
              vertex_stride: u64::from(StaticLayout::STRIDE) * 4,
              index_buffer: Some(statics.geometry.indices.get_buffer()),
              first_index: Some(geometry.first_index),
              transform_buffer: None,
              transform_buffer_offset: None,
            })
            .collect(),
        ),
      })
      .collect();

    for index in 0..self.tlas.get().len() {
      self.tlas[index] = instances.get(index).map(|(structure, transform)| {
        let blas: &wgpu::Blas = Self::find_blas(&self.sectors, &self.models, *structure);

        wgpu::TlasInstance::new(blas, *transform, index as u32, Self::MASK)
      });
    }

    if instances.is_empty() {
      self.tlas[0] = Some(Self::create_placeholder_instance(&self.placeholder));
    }

    encoder.build_acceleration_structures(entries.iter(), std::iter::once(&self.tlas));

    self.built_instances = instances;
    self.built_over = (
      sectors.iter().map(|(sector, ..)| *sector).collect(),
      objects.iter().map(|(_, place, _)| *place).collect(),
    );
    self.swaying = swaying;
    self.version += 1;

    log::debug!(
      "Static trace: {} structures built; {} sectors, {} models, {} instances, {} records",
      entries.len(),
      sectors.len(),
      models.len(),
      self.built_instances.len(),
      records.len()
    );
  }

  /// A built bottom-level structure, of the sectors' or the models'.
  fn find_blas<'b>(
    sectors: &'b HashMap<u32, SectorStructures>,
    models: &'b HashMap<ProxyHandle<StaticModelProxy>, wgpu::Blas>,
    structure: Structure,
  ) -> &'b wgpu::Blas {
    match structure {
      Structure::Baked(sector) => sectors[&sector]
        .baked
        .as_ref()
        .expect("a sector with baked sections has their structure"),
      Structure::Group(sector, group) => &sectors[&sector].groups[group],
      Structure::Model(model) => &models[&model],
    }
  }

  /// Makes the structures of each sector new or moved and each model new since the last build, and lists what each is
  /// built over.
  fn create_structures<'s>(
    &mut self,
    device: &wgpu::Device,
    sectors: &[(u32, &'s [StaticTraceGeometry], &'s [StaticTraceGroup])],
    models: &[(ProxyHandle<StaticModelProxy>, &'s [StaticTraceGeometry])],
  ) -> Vec<PendingBuild<'s>> {
    let mut pending: Vec<PendingBuild<'s>> = Vec::new();
    let mut create = |structure: Structure, geometries: &'s [StaticTraceGeometry]| -> wgpu::Blas {
      let sizes: Vec<wgpu::BlasTriangleGeometrySizeDescriptor> = geometries.iter().map(Self::get_size).collect();
      let blas: wgpu::Blas = device.create_blas(
        &wgpu::CreateBlasDescriptor {
          label: Some("static trace geometry"),
          flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
          update_mode: wgpu::AccelerationStructureUpdateMode::Build,
        },
        wgpu::BlasGeometrySizeDescriptors::Triangles {
          descriptors: sizes.clone(),
        },
      );

      pending.push(PendingBuild {
        structure,
        geometries,
        sizes,
      });

      blas
    };

    for (sector, sections, groups) in sectors {
      let first_index: u32 = sections
        .first()
        .or(groups.first().map(|group| &group.geometry))
        .map_or(0, |geometry| geometry.first_index);

      if self
        .sectors
        .get(sector)
        .is_some_and(|built| built.first_index == first_index)
      {
        continue;
      }

      let baked: Option<wgpu::Blas> = (!sections.is_empty()).then(|| create(Structure::Baked(*sector), sections));
      let groups: Vec<wgpu::Blas> = groups
        .iter()
        .enumerate()
        .map(|(index, group)| create(Structure::Group(*sector, index), std::slice::from_ref(&group.geometry)))
        .collect();

      self.sectors.insert(
        *sector,
        SectorStructures {
          baked,
          groups,
          first_index,
        },
      );
    }

    // A model's geometry never moves in its arena while it stays in the scene: one put in again comes by a new handle.
    for (model, geometries) in models {
      if !self.models.contains_key(model) {
        let blas: wgpu::Blas = create(Structure::Model(*model), geometries);

        self.models.insert(*model, blas);
      }
    }

    pending
  }

  /// The instance table, the records and every top-level instance (its structure and its transform), in one order: an
  /// instance's custom index is its place in the table. A model's records are listed once, its objects' instances
  /// sharing them. And the instances of trees' geometry among them.
  #[allow(clippy::type_complexity)]
  fn list_instances(
    sectors: &[(u32, &[StaticTraceGeometry], &[StaticTraceGroup])],
    models: &[(ProxyHandle<StaticModelProxy>, &[StaticTraceGeometry])],
    objects: &[(ProxyHandle<StaticModelProxy>, u32, &Mat4)],
  ) -> (
    Vec<StaticTraceInstance>,
    Vec<StaticTraceRecord>,
    Vec<(Structure, [f32; 12])>,
    Vec<StaticTraceSwaying>,
  ) {
    let mut table: Vec<StaticTraceInstance> = Vec::new();
    let mut records: Vec<StaticTraceRecord> = Vec::new();
    let mut instances: Vec<(Structure, [f32; 12])> = Vec::new();
    let mut swaying: Vec<StaticTraceSwaying> = Vec::new();

    for (sector, sections, groups) in sectors {
      if !sections.is_empty() {
        table.push(StaticTraceInstance {
          first_record: records.len() as u32,
          place: Self::ORIGIN_PLACE,
          pad: [0; 2],
        });
        instances.push((Structure::Baked(*sector), Self::IDENTITY));
        records.extend(sections.iter().map(StaticTraceRecord::from));
      }

      for (index, group) in groups.iter().enumerate() {
        let first_record: u32 = records.len() as u32;

        records.push(StaticTraceRecord::from(&group.geometry));

        for (offset, transform) in group.transforms.iter().enumerate() {
          let place: u32 = group.first_place + offset as u32;

          if group.geometry.layout == StaticLayout::Tree {
            swaying.push(StaticTraceSwaying {
              index: instances.len() as u32,
              place,
              position: Vec3::new(transform[3], transform[7], transform[11]),
              geometry: group.geometry,
            });
          }

          table.push(StaticTraceInstance {
            first_record,
            place,
            pad: [0; 2],
          });
          instances.push((Structure::Group(*sector, index), *transform));
        }
      }
    }

    let mut first_records: HashMap<ProxyHandle<StaticModelProxy>, u32> = HashMap::with_capacity(models.len());

    for (model, geometries) in models {
      first_records.insert(*model, records.len() as u32);
      records.extend(geometries.iter().map(StaticTraceRecord::from));
    }

    for (model, place, transform) in objects {
      table.push(StaticTraceInstance {
        first_record: first_records[model],
        place: *place,
        pad: [0; 2],
      });
      instances.push((Structure::Model(*model), StaticTraceGroup::to_transform(transform)));
    }

    (table, records, instances, swaying)
  }

  /// Writes the instance table and the records, each buffer made again larger where outgrown.
  fn write_tables(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    table: &[StaticTraceInstance],
    records: &[StaticTraceRecord],
  ) {
    if size_of_val(table) as u64 > self.instances.size() {
      self.instances = Self::create_table::<StaticTraceInstance>(
        device,
        "static trace instances",
        (table.len() as u64).next_power_of_two(),
      );
    }

    if size_of_val(records) as u64 > self.records.size() {
      self.records = Self::create_table::<StaticTraceRecord>(
        device,
        "static trace records",
        (records.len() as u64).next_power_of_two(),
      );
    }

    if !table.is_empty() {
      queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(table));
    }

    if !records.is_empty() {
      queue.write_buffer(&self.records, 0, bytemuck::cast_slice(records));
    }
  }

  fn get_size(geometry: &StaticTraceGeometry) -> wgpu::BlasTriangleGeometrySizeDescriptor {
    wgpu::BlasTriangleGeometrySizeDescriptor {
      vertex_format: wgpu::VertexFormat::Float32x3,
      vertex_count: geometry.vertex_count,
      index_format: Some(wgpu::IndexFormat::Uint32),
      index_count: Some(geometry.index_count),
      flags: if geometry.is_cut_out {
        wgpu::AccelerationStructureGeometryFlags::empty()
      } else {
        wgpu::AccelerationStructureGeometryFlags::OPAQUE
      },
    }
  }

  /// The placeholder's one triangle, unindexed.
  fn get_placeholder_size() -> wgpu::BlasTriangleGeometrySizeDescriptor {
    wgpu::BlasTriangleGeometrySizeDescriptor {
      vertex_format: wgpu::VertexFormat::Float32x3,
      vertex_count: 3,
      index_format: None,
      index_count: None,
      flags: wgpu::AccelerationStructureGeometryFlags::OPAQUE,
    }
  }

  /// The placeholder as an instance of mask zero, which no ray's mask shares a bit with, so nothing ever meets it.
  fn create_placeholder_instance(placeholder: &wgpu::Blas) -> wgpu::TlasInstance {
    wgpu::TlasInstance::new(placeholder, Self::IDENTITY, 0, 0)
  }

  fn create_tlas(device: &wgpu::Device, instances: u32) -> wgpu::Tlas {
    device.create_tlas(&wgpu::CreateTlasDescriptor {
      label: Some("static trace"),
      max_instances: instances,
      flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
      update_mode: wgpu::AccelerationStructureUpdateMode::Build,
    })
  }

  fn create_table<T>(device: &wgpu::Device, label: &'static str, count: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(label),
      size: count.max(1) * size_of::<T>() as u64,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    })
  }
}
