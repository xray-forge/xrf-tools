use std::collections::HashMap;

use xrf_renderer_core::AccelerationStructureBinding;

use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::static_trace_group::StaticTraceGroup;
use crate::scene::static_scene::static_trace_instance::StaticTraceInstance;
use crate::scene::static_scene::static_trace_record::StaticTraceRecord;

/// The level's static geometry as rays are traced against it by hardware ray query: bottom-level acceleration
/// structures read in place from the scene's vertex and index arenas, a sector's deferred baked sections in one and
/// each of its instanced meshes in one of its own; and one top-level structure, an instance for each sector's baked
/// geometry and for each place an instanced mesh stands at. Each instance names an entry of the instance table, its
/// first record and its place, so a hit finds its geometry, its triangle's corners, its surface and its hemisphere
/// (`TraceParameters`). Built again as sectors come and go, whole when an arena it reads grows.
pub struct StaticTrace {
  tlas: wgpu::Tlas,
  /// The key the top-level structure is bound by, a new one each time it is made.
  key: u64,
  instances: wgpu::Buffer,
  records: wgpu::Buffer,
  /// Each sector's structures, by its index in the level.
  sectors: HashMap<u32, SectorStructures>,
  /// The scene's contents and the generations of the arenas read, as last built over.
  built: Option<(usize, [u64; StaticLayout::COUNT + 1])>,
}

/// One sector's bottom-level structures, and the first index it was built over, which a sector moved in its arena
/// changes: its baked sections', where it has any, and each instanced mesh's.
struct SectorStructures {
  baked: Option<wgpu::Blas>,
  groups: Vec<wgpu::Blas>,
  first_index: u32,
}

/// A bottom-level structure waiting to be built over its geometries.
struct PendingBuild<'s> {
  sector: u32,
  /// None for the sector's baked sections, else the instanced mesh's index.
  group: Option<usize>,
  geometries: &'s [StaticTraceGeometry],
  sizes: Vec<wgpu::BlasTriangleGeometrySizeDescriptor>,
}

impl StaticTrace {
  /// Instances the top-level structure holds at first; it is made again twice as large when outgrown.
  const INSTANCES: u32 = 64;

  /// Records and instance table entries their buffers hold at first.
  const RECORDS: u64 = 1024;

  /// An instance's transform where the geometry stands in renderer space already.
  const IDENTITY: [f32; 12] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

  /// The place every sector's baked geometry stands at: the scene's origin, whose hemisphere is the vertices' own.
  const ORIGIN_PLACE: u32 = 0;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let tlas: wgpu::Tlas = Self::create_tlas(device, Self::INSTANCES);
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("static trace"),
    });

    // Built empty, so it is bound built before any sector is in.
    encoder.build_acceleration_structures(std::iter::empty(), std::iter::once(&tlas));
    queue.submit([encoder.finish()]);

    Self {
      tlas,
      key: AccelerationStructureBinding::next_key(),
      instances: Self::create_table::<StaticTraceInstance>(device, "static trace instances", Self::RECORDS),
      records: Self::create_table::<StaticTraceRecord>(device, "static trace records", Self::RECORDS),
      sectors: HashMap::new(),
      built: None,
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

  /// Each geometry's record, an instance's in a run its table entry names the first of.
  pub fn get_records(&self) -> &wgpu::Buffer {
    &self.records
  }

  /// Builds what changed in the scene since the last call into `encoder`: the structures of each sector come in or
  /// moved, every one where an arena grew, and the top level over them all.
  pub fn update(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    statics: &StaticScene,
  ) {
    let generations: [u64; StaticLayout::COUNT + 1] = [
      statics.words[0].get_generation(),
      statics.words[1].get_generation(),
      statics.words[2].get_generation(),
      statics.indices.get_generation(),
    ];
    let built: (usize, [u64; StaticLayout::COUNT + 1]) = (statics.get_contents(), generations);

    if self.built.is_some_and(|(_, held)| held != generations) {
      self.sectors.clear();
    }

    if self.built != Some(built) {
      self.built = Some(built);
      self.build(device, queue, encoder, statics);
    }
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

    self
      .sectors
      .retain(|sector, _| sectors.iter().any(|(present, ..)| present == sector));

    let pending: Vec<PendingBuild<'_>> = self.create_structures(device, &sectors);
    let (table, records, instances) = self.list_instances(&sectors);

    self.write_tables(device, queue, &table, &records);

    if instances.len() > self.tlas.get().len() {
      self.tlas = Self::create_tlas(device, (instances.len() as u32).next_power_of_two());
      self.key = AccelerationStructureBinding::next_key();
    }

    let entries: Vec<wgpu::BlasBuildEntry<'_>> = pending
      .iter()
      .map(|build| {
        let structures: &SectorStructures = &self.sectors[&build.sector];

        wgpu::BlasBuildEntry {
          blas: match build.group {
            None => structures.baked.as_ref().expect("a sector with baked sections has their structure"),
            Some(group) => &structures.groups[group],
          },
          geometry: wgpu::BlasGeometries::TriangleGeometries(
            build
              .geometries
              .iter()
              .zip(&build.sizes)
              .map(|(geometry, size)| wgpu::BlasTriangleGeometry {
                size,
                // Each vertex's position is its first three words, so the arena is read in place.
                vertex_buffer: statics.words[geometry.layout.get_index()].get_buffer(),
                first_vertex: geometry.vertex_start,
                vertex_stride: u64::from(StaticLayout::STRIDE) * 4,
                index_buffer: Some(statics.indices.get_buffer()),
                first_index: Some(geometry.first_index),
                transform_buffer: None,
                transform_buffer_offset: None,
              })
              .collect(),
          ),
        }
      })
      .collect();
    for index in 0..self.tlas.get().len() {
      self.tlas[index] = instances.get(index).map(|(sector, group, transform)| {
        let structures: &SectorStructures = &self.sectors[sector];
        let blas: &wgpu::Blas = match group {
          None => structures.baked.as_ref().expect("a sector with baked sections has their structure"),
          Some(group) => &structures.groups[*group],
        };

        wgpu::TlasInstance::new(blas, *transform, index as u32, 0xff)
      });
    }

    encoder.build_acceleration_structures(entries.iter(), std::iter::once(&self.tlas));

    log::info!(
      "Static trace: {} structures built; {} sectors, {} instances, {} records",
      entries.len(),
      sectors.len(),
      instances.len(),
      records.len()
    );
  }

  /// Makes the structures of each sector new or moved since the last build, and lists what each is built over.
  fn create_structures<'s>(
    &mut self,
    device: &wgpu::Device,
    sectors: &[(u32, &'s [StaticTraceGeometry], &'s [StaticTraceGroup])],
  ) -> Vec<PendingBuild<'s>> {
    let mut pending: Vec<PendingBuild<'s>> = Vec::new();

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

      let mut create = |group: Option<usize>, geometries: &'s [StaticTraceGeometry]| -> wgpu::Blas {
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
          sector: *sector,
          group,
          geometries,
          sizes,
        });

        blas
      };
      let baked: Option<wgpu::Blas> = (!sections.is_empty()).then(|| create(None, sections));
      let groups: Vec<wgpu::Blas> = groups
        .iter()
        .enumerate()
        .map(|(index, group)| create(Some(index), std::slice::from_ref(&group.geometry)))
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

    pending
  }

  /// The instance table, the records and every top-level instance (its sector, its instanced mesh or none for the
  /// baked sections, its transform), in one order: an instance's custom index is its place in the table.
  #[allow(clippy::type_complexity)]
  fn list_instances(
    &self,
    sectors: &[(u32, &[StaticTraceGeometry], &[StaticTraceGroup])],
  ) -> (Vec<StaticTraceInstance>, Vec<StaticTraceRecord>, Vec<(u32, Option<usize>, [f32; 12])>) {
    let mut table: Vec<StaticTraceInstance> = Vec::new();
    let mut records: Vec<StaticTraceRecord> = Vec::new();
    let mut instances: Vec<(u32, Option<usize>, [f32; 12])> = Vec::new();

    for (sector, sections, groups) in sectors {
      if !sections.is_empty() {
        table.push(StaticTraceInstance {
          first_record: records.len() as u32,
          place: Self::ORIGIN_PLACE,
          pad: [0; 2],
        });
        instances.push((*sector, None, Self::IDENTITY));
        records.extend(sections.iter().map(StaticTraceRecord::from));
      }

      for (index, group) in groups.iter().enumerate() {
        let first_record: u32 = records.len() as u32;

        records.push(StaticTraceRecord::from(&group.geometry));

        for (offset, transform) in group.transforms.iter().enumerate() {
          table.push(StaticTraceInstance {
            first_record,
            place: group.first_place + offset as u32,
            pad: [0; 2],
          });
          instances.push((*sector, Some(index), *transform));
        }
      }
    }

    (table, records, instances)
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
      self.records =
        Self::create_table::<StaticTraceRecord>(device, "static trace records", (records.len() as u64).next_power_of_two());
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
