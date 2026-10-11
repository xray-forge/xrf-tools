use std::collections::HashMap;

use xrf_renderer_core::AccelerationStructureBinding;

use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::static_trace_record::StaticTraceRecord;

/// The level's static geometry as rays are traced against it by hardware ray query: a bottom-level acceleration
/// structure a sector over its deferred baked sections, read in place from the scene's baked vertices and indices, and
/// one top-level structure with an instance a sector, each naming its first record, so a hit finds its geometry, its
/// triangle's corners and its surface (`TraceParameters`). Built again as sectors come and go, whole when an arena it
/// reads grows.
pub struct StaticTrace {
  tlas: wgpu::Tlas,
  /// The key the top-level structure is bound by, a new one each time it is made.
  key: u64,
  records: wgpu::Buffer,
  /// Each sector's structure, by its index in the level.
  sectors: HashMap<u32, SectorStructure>,
  /// The scene's contents and the generations of the arenas read, as last built over.
  built: Option<(usize, u64, u64)>,
}

/// One sector's bottom-level structure and the first index it was built over, which a sector moved in its arena
/// changes.
struct SectorStructure {
  blas: wgpu::Blas,
  first_index: u32,
}

impl StaticTrace {
  /// Instances the top-level structure holds at first; it is made again twice as large when outgrown.
  const INSTANCES: u32 = 64;

  /// Records the record buffer holds at first.
  const RECORDS: u64 = 1024;

  /// An instance's transform: the sectors' geometry stands in renderer space already.
  const IDENTITY: [f32; 12] = [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

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
      records: Self::create_records(device, Self::RECORDS),
      sectors: HashMap::new(),
      built: None,
    }
  }

  /// The top-level structure, as a pass binds it.
  pub fn get_structure(&self) -> AccelerationStructureBinding<'_> {
    AccelerationStructureBinding::new(&self.tlas, self.key)
  }

  /// Each geometry's record, a sector's in a run its instance names the first of.
  pub fn get_records(&self) -> &wgpu::Buffer {
    &self.records
  }

  /// Builds what changed in the scene since the last call into `encoder`: a structure for each sector come in or moved,
  /// every one where an arena grew, and the top level over them all.
  pub fn update(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    encoder: &mut wgpu::CommandEncoder,
    statics: &StaticScene,
  ) {
    let built: (usize, u64, u64) = (
      statics.get_contents(),
      statics.words[StaticLayout::Baked.get_index()].get_generation(),
      statics.indices.get_generation(),
    );

    if self.built.is_some_and(|it| (it.1, it.2) != (built.1, built.2)) {
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
    let sectors: Vec<(u32, &[StaticTraceGeometry])> = statics
      .list_trace_geometries()
      .filter(|(_, geometries)| !geometries.is_empty())
      .collect();

    self
      .sectors
      .retain(|sector, _| sectors.iter().any(|(present, _)| present == sector));

    // Each new or moved sector's structure, sized by its sections.
    let mut sizes: Vec<(usize, Vec<wgpu::BlasTriangleGeometrySizeDescriptor>)> = Vec::new();

    for (at, (sector, geometries)) in sectors.iter().enumerate() {
      let first_index: u32 = geometries[0].first_index;

      if self
        .sectors
        .get(sector)
        .is_some_and(|built| built.first_index == first_index)
      {
        continue;
      }

      let descriptors: Vec<wgpu::BlasTriangleGeometrySizeDescriptor> =
        geometries.iter().map(Self::get_size).collect();
      let blas: wgpu::Blas = device.create_blas(
        &wgpu::CreateBlasDescriptor {
          label: Some("static trace sector"),
          flags: wgpu::AccelerationStructureFlags::PREFER_FAST_TRACE,
          update_mode: wgpu::AccelerationStructureUpdateMode::Build,
        },
        wgpu::BlasGeometrySizeDescriptors::Triangles {
          descriptors: descriptors.clone(),
        },
      );

      self.sectors.insert(*sector, SectorStructure { blas, first_index });
      sizes.push((at, descriptors));
    }

    // Each vertex's position is its first three words, so the arena is read in place.
    let vertices: &wgpu::Buffer = statics.words[StaticLayout::Baked.get_index()].get_buffer();
    let indices: &wgpu::Buffer = statics.indices.get_buffer();
    let entries: Vec<wgpu::BlasBuildEntry<'_>> = sizes
      .iter()
      .map(|(at, descriptors)| {
        let (sector, geometries) = &sectors[*at];

        wgpu::BlasBuildEntry {
          blas: &self.sectors[sector].blas,
          geometry: wgpu::BlasGeometries::TriangleGeometries(
            geometries
              .iter()
              .zip(descriptors)
              .map(|(geometry, size)| wgpu::BlasTriangleGeometry {
                size,
                vertex_buffer: vertices,
                first_vertex: geometry.vertex_start,
                vertex_stride: u64::from(StaticLayout::STRIDE) * 4,
                index_buffer: Some(indices),
                first_index: Some(geometry.first_index),
                transform_buffer: None,
                transform_buffer_offset: None,
              })
              .collect(),
          ),
        }
      })
      .collect();

    // The records, a sector's in a run its instance names the first of.
    let mut records: Vec<StaticTraceRecord> = Vec::new();
    let mut instances: Vec<(u32, u32)> = Vec::new();

    for (sector, geometries) in &sectors {
      instances.push((*sector, records.len() as u32));
      records.extend(geometries.iter().map(StaticTraceRecord::from));
    }

    if (records.len() * size_of::<StaticTraceRecord>()) as u64 > self.records.size() {
      self.records = Self::create_records(device, (records.len() as u64).next_power_of_two());
    }

    if !records.is_empty() {
      queue.write_buffer(&self.records, 0, bytemuck::cast_slice(&records));
    }

    if instances.len() > self.tlas.get().len() {
      self.tlas = Self::create_tlas(device, (instances.len() as u32).next_power_of_two());
      self.key = AccelerationStructureBinding::next_key();
    }

    for index in 0..self.tlas.get().len() {
      self.tlas[index] = instances.get(index).map(|(sector, first)| {
        wgpu::TlasInstance::new(&self.sectors[sector].blas, Self::IDENTITY, *first, 0xff)
      });
    }

    encoder.build_acceleration_structures(entries.iter(), std::iter::once(&self.tlas));

    let triangles: u32 = sectors
      .iter()
      .flat_map(|(_, geometries)| geometries.iter())
      .map(|geometry| geometry.index_count / 3)
      .sum();

    log::info!(
      "Static trace: {} sector structures built; {} sectors, {triangles} triangles traced",
      entries.len(),
      sectors.len()
    );
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

  fn create_records(device: &wgpu::Device, count: u64) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("static trace records"),
      size: count * size_of::<StaticTraceRecord>() as u64,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    })
  }
}
