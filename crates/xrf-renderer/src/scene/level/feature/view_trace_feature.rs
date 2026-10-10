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
use crate::scene::level::grass_cards::GrassCards;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_trace::StaticTrace;
use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::static_trace_swaying::StaticTraceSwaying;

/// A view's own top level, where what its trace meets moves with the view: the nearest instances of trees' geometry
/// within `REACH` metres, `MOST` of them and `MOST_VERTICES` vertices at most, swayed by the view's wind each frame and
/// their structures built over them again, standing in their unswayed instances' place; and the grass's cards the view
/// stands (`GrassCards`), one more instance after the scene's. The scene's top level where neither stands.
#[derive(Default)]
pub struct ViewTraceFeature {
  trace: Option<ViewTrace>,
}

/// What a view keeps for its own top level: the top level and the key it is bound by, each swaying instance with its
/// structure, their slots for the sway pass and how many, and the vertices it writes; both buffers are kept as they
/// shrink, so a frame's own part of them is bound alone.
struct ViewTrace {
  tlas: wgpu::Tlas,
  key: u64,
  swayed: Vec<SwayedInstance>,
  slots: wgpu::Buffer,
  slot_count: u32,
  vertices: wgpu::Buffer,
  vertex_count: u32,
}

/// One instance the view sways: its index in the top level, its geometry and its own structure.
struct SwayedInstance {
  index: u32,
  geometry: StaticTraceGeometry,
  blas: wgpu::Blas,
}

impl ViewTraceFeature {
  /// Metres from the camera within which a tree's instance sways: past it a sway moves a shadow by less than its
  /// penumbra.
  const REACH: f32 = 60.0;
  /// Instances that sway at most, and their vertices.
  const MOST: usize = 64;
  const MOST_VERTICES: u32 = 1 << 20;

  /// Takes this frame's swaying trees about the camera at `camera`, where the scene's trace stands (`trace`) and the
  /// view traces (`is_traced`): the nearest chosen, a structure made for each newly chosen, the sway pass's slots
  /// written and the view's top level filled, with the grass's `cards` where the view stands them.
  pub fn prepare(
    &mut self,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    (trace, cards): (Option<&StaticTrace>, Option<&wgpu::Blas>),
    camera: Vec3,
    is_traced: bool,
  ) {
    let Some(trace) = trace.filter(|_| is_traced) else {
      self.trace = None;

      return;
    };
    let chosen: Vec<StaticTraceSwaying> = Self::choose(trace.get_swaying(), camera);

    if chosen.is_empty() && cards.is_none() {
      self.trace = None;

      return;
    }

    let previous: Option<ViewTrace> = self.trace.take();
    let mut kept: HashMap<(u32, u32), wgpu::Blas> = previous
      .iter()
      .flat_map(|trace| &trace.swayed)
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
    // The scene's instances, then the grass's.
    let instances: usize = trace.get_instance_count() + 1;
    let (mut tlas, key, slot_buffer, vertices) = match previous {
      Some(trace) if trace.tlas.get().len() >= instances => (trace.tlas, trace.key, trace.slots, trace.vertices),
      Some(trace) => (
        Self::create_tlas(device, instances),
        AccelerationStructureBinding::next_key(),
        trace.slots,
        trace.vertices,
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
    tlas[instances - 1] =
      cards.map(|blas| wgpu::TlasInstance::new(blas, StaticTrace::IDENTITY, GrassCards::INSTANCE, GrassCards::MASK));

    self.trace = Some(ViewTrace {
      tlas,
      key,
      swayed,
      slots: slot_buffer,
      slot_count: slots.len() as u32,
      vertices,
      vertex_count: first_out,
    });
  }

  /// Sways the chosen trees' vertices and builds their structures and the view's top level over them and the grass's
  /// cards, built before it, answering the top level the view's trace binds; none where neither stands, the scene's
  /// standing.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    passes: LevelPasses<'a>,
    statics: &StaticSceneHandles,
  ) -> Option<AccelerationStructureBinding<'a>> {
    let trace: &'a ViewTrace = self.trace.as_ref()?;
    let vertices: GraphBuffer = bindings.import_buffer(graph, "trace swayed vertices", &trace.vertices);
    let slots: GraphBuffer = bindings.import_buffer(graph, "trace sway slots", &trace.slots);
    let indices: GraphBuffer = statics.indices;
    let parameters: TraceSwayParameters = TraceSwayParameters {
      words: StorageArray::new(statics.words[StaticLayout::Tree.get_index()]),
      places: StorageArray::new(statics.places),
      surfaces: StorageArray::new(statics.surfaces),
      wind: statics.wind,
      // The frame's slots alone: the sway finds a vertex's slot by searching all it binds.
      sway_slots: StorageArray::new_range(
        slots,
        0,
        u64::from(trace.slot_count) * size_of::<TraceSwaySlot>() as u64,
      ),
      swayed_vertices: StorageArrayMut::new_range(vertices, 0, u64::from(trace.vertex_count) * 12),
    };
    let count: u32 = trace.vertex_count;

    if count > 0 {
      graph
        .add_compute_pass("trace sway")
        .parameters(&parameters)
        .record(move |context| passes.trace_sway.record(context, &parameters, count));
    }
    graph
      .add_encoder_pass("trace sway build")
      .buffer(vertices, GraphBufferAccess::StorageRead)
      .buffer(indices, GraphBufferAccess::StorageRead)
      .keep()
      .record(move |context| {
        let (vertices, indices) = (context.get_buffer(vertices), context.get_buffer(indices));
        let mut first_vertex: u32 = 0;
        let sizes: Vec<wgpu::BlasTriangleGeometrySizeDescriptor> =
          trace.swayed.iter().map(|it| Self::get_size(&it.geometry)).collect();
        let entries: Vec<wgpu::BlasBuildEntry<'_>> = trace
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
          .build_acceleration_structures(entries.iter(), std::iter::once(&trace.tlas));
      });

    Some(AccelerationStructureBinding::new(&trace.tlas, trace.key))
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

impl ViewFeature for ViewTraceFeature {
  /// Its swayed vertices, slots and structures.
  fn get_memory(&self) -> u64 {
    self
      .trace
      .as_ref()
      .map_or(0, |trace| trace.vertices.size() + trace.slots.size())
  }

  fn release(&mut self) {
    self.trace = None;
  }
}
