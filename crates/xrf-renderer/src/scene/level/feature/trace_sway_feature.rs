use std::collections::HashMap;

use glam::Vec3;
use xrf_renderer_core::{
  AccelerationStructureBinding, FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, StorageArray,
  StorageArrayMut,
};

use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::trace_sway_parameters::TraceSwayParameters;
use crate::pass::trace_sway_slot::TraceSwaySlot;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_trace::StaticTrace;
use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::static_trace_swaying::StaticTraceSwaying;

/// The trees a view's trace stands swayed: the nearest instances of trees' geometry within `REACH` metres, `MOST` of
/// them and `MOST_VERTICES` vertices at most, their vertices swayed by the view's wind each frame and their structures
/// built over them again, standing in the view's own top level in their unswayed instances' place; the scene's top
/// level where none sway.
#[derive(Default)]
pub struct TraceSwayFeature {
  sway: Option<TraceSway>,
}

/// What a view keeps to sway its trace's trees: its top level and the key it is bound by, each swaying instance with its
/// structure, their slots for the sway pass, and the vertices it writes.
struct TraceSway {
  tlas: wgpu::Tlas,
  key: u64,
  swayed: Vec<SwayedInstance>,
  slots: wgpu::Buffer,
  vertices: wgpu::Buffer,
  vertex_count: u32,
}

/// One instance the view sways: its index in the top level, its geometry and its own structure.
struct SwayedInstance {
  index: u32,
  geometry: StaticTraceGeometry,
  blas: wgpu::Blas,
}

impl TraceSwayFeature {
  /// Metres from the camera within which a tree's instance sways: past it a sway moves a shadow by less than its
  /// penumbra.
  const REACH: f32 = 60.0;
  /// Instances that sway at most, and their vertices.
  const MOST: usize = 64;
  const MOST_VERTICES: u32 = 1 << 20;

  /// Takes this frame's swaying trees about the camera at `camera`, where the scene's trace stands (`trace`) and the
  /// view traces (`is_traced`): the nearest chosen, a structure made for each newly chosen, the sway pass's slots
  /// written and the view's top level filled.
  pub fn prepare(
    &mut self,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    trace: Option<&StaticTrace>,
    camera: Vec3,
    is_traced: bool,
  ) {
    let Some(trace) = trace.filter(|_| is_traced) else {
      self.sway = None;

      return;
    };
    let chosen: Vec<StaticTraceSwaying> = Self::choose(trace.get_swaying(), camera);

    if chosen.is_empty() {
      self.sway = None;

      return;
    }

    let previous: Option<TraceSway> = self.sway.take();
    let mut kept: HashMap<(u32, u32), wgpu::Blas> = previous
      .iter()
      .flat_map(|sway| &sway.swayed)
      .map(|swayed| ((swayed.index, swayed.geometry.first_index), swayed.blas.clone()))
      .collect();
    let swayed: Vec<SwayedInstance> = chosen
      .iter()
      .map(|swaying| SwayedInstance {
        index: swaying.index,
        geometry: swaying.geometry,
        blas: kept
          .remove(&(swaying.index, swaying.geometry.first_index))
          .unwrap_or_else(|| Self::create_blas(device, &swaying.geometry)),
      })
      .collect();
    let mut first_out: u32 = 0;
    let slots: Vec<TraceSwaySlot> = chosen
      .iter()
      .map(|swaying| {
        let slot: TraceSwaySlot = TraceSwaySlot {
          vertex_start: swaying.geometry.vertex_start,
          vertex_count: swaying.geometry.vertex_count,
          place: swaying.place,
          surface: swaying.geometry.surface,
          first_out,
        };

        first_out += swaying.geometry.vertex_count;

        slot
      })
      .collect();
    let instances: usize = trace.get_instance_count();
    let (mut tlas, key, slot_buffer, vertices) = match previous {
      Some(sway) if sway.tlas.get().len() >= instances => (sway.tlas, sway.key, sway.slots, sway.vertices),
      Some(sway) => (
        Self::create_tlas(device, instances),
        AccelerationStructureBinding::next_key(),
        sway.slots,
        sway.vertices,
      ),
      None => (
        Self::create_tlas(device, instances),
        AccelerationStructureBinding::next_key(),
        Self::create_buffer(device, "trace sway slots", 0, wgpu::BufferUsages::COPY_DST),
        Self::create_buffer(device, "trace swayed vertices", 0, wgpu::BufferUsages::BLAS_INPUT),
      ),
    };
    let slots_size: u64 = size_of_val(slots.as_slice()) as u64;
    let vertices_size: u64 = u64::from(first_out) * 12;
    let slot_buffer: wgpu::Buffer = if slot_buffer.size() < slots_size {
      Self::create_buffer(device, "trace sway slots", slots_size, wgpu::BufferUsages::COPY_DST)
    } else {
      slot_buffer
    };
    let vertices: wgpu::Buffer = if vertices.size() < vertices_size {
      Self::create_buffer(
        device,
        "trace swayed vertices",
        vertices_size,
        wgpu::BufferUsages::BLAS_INPUT,
      )
    } else {
      vertices
    };

    queue.write_buffer(&slot_buffer, 0, bytemuck::cast_slice(&slots));

    let replaced: HashMap<u32, &wgpu::Blas> = swayed.iter().map(|it| (it.index, &it.blas)).collect();

    trace.fill_view_tlas(&mut tlas, &replaced);

    self.sway = Some(TraceSway {
      tlas,
      key,
      swayed,
      slots: slot_buffer,
      vertices,
      vertex_count: first_out,
    });
  }

  /// Sways the chosen trees' vertices and builds their structures and the view's top level over them, answering the
  /// top level the view's trace binds; none where none sway, the scene's standing.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    statics: &StaticSceneHandles,
  ) -> Option<AccelerationStructureBinding<'a>> {
    let sway: &'a TraceSway = self.sway.as_ref()?;
    let vertices: GraphBuffer = bindings.import_buffer(graph, "trace swayed vertices", &sway.vertices);
    let slots: GraphBuffer = bindings.import_buffer(graph, "trace sway slots", &sway.slots);
    let indices: GraphBuffer = statics.indices;
    let parameters: TraceSwayParameters = TraceSwayParameters {
      words: StorageArray::new(statics.words[StaticLayout::Tree.get_index()]),
      places: StorageArray::new(statics.places),
      surfaces: StorageArray::new(statics.surfaces),
      wind: statics.wind,
      sway_slots: StorageArray::new(slots),
      swayed_vertices: StorageArrayMut::new(vertices),
    };
    let count: u32 = sway.vertex_count;

    graph
      .add_compute_pass("trace sway")
      .parameters(&parameters)
      .record(move |context| passes.trace_sway.record(context, &parameters, count));
    graph
      .add_encoder_pass("trace sway build")
      .buffer(vertices, GraphBufferAccess::StorageRead)
      .buffer(indices, GraphBufferAccess::StorageRead)
      .keep()
      .record(move |context| {
        let (vertices, indices) = (context.get_buffer(vertices), context.get_buffer(indices));
        let mut first_vertex: u32 = 0;
        let sizes: Vec<wgpu::BlasTriangleGeometrySizeDescriptor> =
          sway.swayed.iter().map(|it| Self::get_size(&it.geometry)).collect();
        let entries: Vec<wgpu::BlasBuildEntry<'_>> = sway
          .swayed
          .iter()
          .zip(&sizes)
          .map(|(swayed, size)| {
            let geometry: wgpu::BlasTriangleGeometry<'_> = wgpu::BlasTriangleGeometry {
              size,
              vertex_buffer: vertices,
              first_vertex,
              vertex_stride: 12,
              index_buffer: Some(indices),
              first_index: Some(swayed.geometry.first_index),
              transform_buffer: None,
              transform_buffer_offset: None,
            };

            first_vertex += swayed.geometry.vertex_count;

            wgpu::BlasBuildEntry {
              blas: &swayed.blas,
              geometry: wgpu::BlasGeometries::TriangleGeometries(vec![geometry]),
            }
          })
          .collect();

        context
          .get_encoder()
          .build_acceleration_structures(entries.iter(), std::iter::once(&sway.tlas));
      });

    Some(AccelerationStructureBinding::new(&sway.tlas, sway.key))
  }

  /// The nearest instances within reach, as many as the limits let in.
  fn choose(swaying: &[StaticTraceSwaying], camera: Vec3) -> Vec<StaticTraceSwaying> {
    let mut near: Vec<(f32, StaticTraceSwaying)> = swaying
      .iter()
      .map(|it| (it.position.distance_squared(camera), *it))
      .filter(|(distance, _)| *distance <= Self::REACH * Self::REACH)
      .collect();

    near.sort_by(|a, b| a.0.total_cmp(&b.0));

    let mut vertices: u32 = 0;

    near
      .into_iter()
      .map(|(_, it)| it)
      .take_while(|it| {
        vertices += it.geometry.vertex_count;

        vertices <= Self::MOST_VERTICES
      })
      .take(Self::MOST)
      .collect()
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

  fn create_blas(device: &wgpu::Device, geometry: &StaticTraceGeometry) -> wgpu::Blas {
    device.create_blas(
      &wgpu::CreateBlasDescriptor {
        label: Some("trace swayed tree"),
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_BUILD,
        update_mode: wgpu::AccelerationStructureUpdateMode::Build,
      },
      wgpu::BlasGeometrySizeDescriptors::Triangles {
        descriptors: vec![Self::get_size(geometry)],
      },
    )
  }

  fn create_tlas(device: &wgpu::Device, instances: usize) -> wgpu::Tlas {
    device.create_tlas(&wgpu::CreateTlasDescriptor {
      label: Some("trace swayed"),
      max_instances: (instances.max(1) as u32).next_power_of_two(),
      flags: wgpu::AccelerationStructureFlags::PREFER_FAST_BUILD,
      update_mode: wgpu::AccelerationStructureUpdateMode::Build,
    })
  }

  fn create_buffer(device: &wgpu::Device, label: &'static str, size: u64, usage: wgpu::BufferUsages) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(label),
      size: size.max(16).next_power_of_two(),
      usage: wgpu::BufferUsages::STORAGE | usage,
      mapped_at_creation: false,
    })
  }
}

impl ViewFeature for TraceSwayFeature {
  /// Its swayed vertices, slots and structures.
  fn get_memory(&self) -> u64 {
    self
      .sway
      .as_ref()
      .map_or(0, |sway| sway.vertices.size() + sway.slots.size())
  }

  fn release(&mut self) {
    self.sway = None;
  }
}
