use std::collections::HashMap;

use glam::Vec3;
use xrf_renderer_core::{
  AccelerationStructureBinding, GraphBuffer, GraphBufferAccess, GraphDeclaration, StorageArray, StorageArrayMut,
};

use crate::feature::lighting::lights::lights_feature::LightsFeature;
use crate::feature::scene::drawable::drawable_kind_traced::DrawableKindTraced;
use crate::feature::scene::drawable::drawable_trace_instance::DrawableTraceInstance;
use crate::feature::scene::grass::trace::traced_grass::TracedGrass;
use crate::feature::tracing::view_trace::sway::trace_sway_parameters::TraceSwayParameters;
use crate::feature::tracing::view_trace::sway::trace_sway_slot::TraceSwaySlot;
use crate::feature::tracing::view_trace::trace_parameters::TraceParameters;
use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::trace::static_trace::StaticTrace;
use crate::scene::static_scene::trace::static_trace_geometry::StaticTraceGeometry;
use crate::scene::static_scene::trace::static_trace_group::StaticTraceGroup;
use crate::scene::static_scene::trace::static_trace_swaying::StaticTraceSwaying;

/// A view's own top level, where what its trace meets moves with the view: the nearest instances of trees' geometry
/// within `REACH` metres, `MOST` of them and `MOST_VERTICES` vertices at most, swayed by the view's wind each frame in
/// their trees' own space and their structures built over them again, standing in their unswayed instances' place and
/// at their transforms; and the drawable kinds' instances after the scene's (`DrawableTraceInstance`: the grass's cards
/// the view stands). The scene's top level where neither stands.
#[derive(Default)]
pub(crate) struct ViewTraceFeature {
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

  /// Takes this frame's swaying trees about the camera, where the scene's trace stands (`trace`) and the view traces
  /// the scene: the nearest chosen while the wind sways them (`choose`), a structure made for each newly chosen, the
  /// sway pass's slots written and the view's top level filled, the drawable `kinds`' instances after the scene's (the
  /// grass's cards where the view stands them). With no wind the trees stand as the scene's trace holds them, its shared
  /// structures quicker to trace than one refitted a tree, and with no kinds either the view traces the scene's own top
  /// level.
  pub(crate) fn prepare(
    &mut self,
    &ViewPrepare {
      device,
      passes,
      view,
      options,
      ..
    }: &ViewPrepare<'_>,
    encoder: &mut FrameEncoder,
    trace: Option<&StaticTrace>,
    sway_amplitude: f32,
    kinds: &[DrawableTraceInstance<'_>],
  ) {
    let camera: Vec3 = view.position;
    let is_traced: bool = options.is_scene_traced() && passes.tracing.scene_trace.is_traced();

    let Some(trace) = trace.filter(|_| is_traced) else {
      self.trace = None;

      return;
    };
    let chosen: Vec<StaticTraceSwaying> = Self::choose(trace.get_swaying(), camera, sway_amplitude);

    if chosen.is_empty() && kinds.is_empty() {
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
          to_local: StaticTraceGroup::unpack_transform(&swaying.transform).inverse(),
          vertex_start: swaying.geometry.vertex_start,
          vertex_count: swaying.geometry.vertex_count,
          place: swaying.place,
          surface: swaying.geometry.surface,
          first_out,
          _pad: [0; 3],
        };

        first_out += swaying.geometry.vertex_count;

        slot
      })
      .collect();
    // The scene's instances, then the kinds'.
    let scene_instances: usize = trace.get_instance_count();
    let instances: usize = scene_instances + kinds.len();
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

    encoder.write_buffer(&slot_buffer, 0, bytemuck::cast_slice(&slots));

    let replaced: HashMap<u32, &wgpu::Blas> = swayed.iter().map(|it| (it.index, &it.blas)).collect();

    trace.fill_view_tlas(&mut tlas, &replaced);

    // A kind's structure stands in the renderer's space already.
    for (index, kind) in kinds.iter().enumerate() {
      tlas[scene_instances + index] = Some(wgpu::TlasInstance::new(
        kind.blas,
        StaticTrace::IDENTITY,
        kind.custom_index,
        kind.mask,
      ));
    }

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
  fn add_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
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
        .record(move |context| passes.tracing.trace_sway.record(context, &parameters, count));
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
          trace.swayed.iter().map(|it| it.geometry.get_size()).collect();
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

  /// The nearest instances within reach, as many as the limits let in, while the wind sways them (`sway_amplitude`
  /// over nought); none while it is still, each then standing as the scene's trace holds it.
  pub(crate) fn choose(swaying: &[StaticTraceSwaying], camera: Vec3, sway_amplitude: f32) -> Vec<StaticTraceSwaying> {
    if sway_amplitude <= 0.0 {
      return Vec::new();
    }

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

  fn create_blas(device: &wgpu::Device, geometry: &StaticTraceGeometry) -> wgpu::Blas {
    device.create_blas(
      &wgpu::CreateBlasDescriptor {
        label: Some("trace swayed tree"),
        flags: wgpu::AccelerationStructureFlags::PREFER_FAST_BUILD | wgpu::AccelerationStructureFlags::ALLOW_UPDATE,
        update_mode: wgpu::AccelerationStructureUpdateMode::PreferUpdate,
      },
      wgpu::BlasGeometrySizeDescriptors::Triangles {
        descriptors: vec![geometry.get_size()],
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

  /// The scene's trace where the view traces rays against it this frame: the scene holds one and the device traces.
  pub(crate) fn get_scene_trace<'a>(scene: &'a LevelScene, passes: LevelPasses<'_>) -> Option<&'a StaticTrace> {
    scene.trace.as_ref().filter(|_| passes.tracing.scene_trace.is_traced())
  }

  /// The level's geometry a ray the screen does not answer is traced against, as one parameter set a frame which every
  /// consumer binds (the reflections, the water), where the device traces rays; with the drawable `kinds`' parts, which
  /// each kind recorded over the same trace. The trace's shaders bind the grass's by name, so it traces nothing without
  /// it.
  pub(crate) fn add_scene_trace<'a>(
    &'a self,
    scene: &'a LevelScene,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    lights: &'a LightsFeature,
    kinds: &[DrawableKindTraced],
  ) -> Option<TracedScene<'a>> {
    let grass: Option<TracedGrass> = kinds.iter().find_map(DrawableKindTraced::get_grass);
    let ViewRecord {
      passes,
      textures: texture_group,
      statics,
      lit,
      ..
    } = record;

    // The view's own top level where its nearest trees stand swayed or a kind's instance stands, the scene's otherwise.
    let swayed: Option<AccelerationStructureBinding<'a>> =
      self.add_passes(GraphDeclaration::new(graph, bindings, runtime), passes, &statics);

    Self::get_scene_trace(scene, passes)
      .zip(grass)
      .map(|(trace, grass)| TracedScene {
        trace: TraceParameters {
          trace_scene: swayed.unwrap_or_else(|| trace.get_structure()),
          trace_instances: StorageArray::new(bindings.import_buffer(
            &mut *graph,
            "static trace instances",
            trace.get_instances(),
          )),
          trace_records: StorageArray::new(bindings.import_buffer(
            &mut *graph,
            "static trace records",
            trace.get_records(),
          )),
          trace_places: StorageArray::new(statics.places),
          trace_baked_words: StorageArray::new(statics.words[StaticLayout::Baked.get_index()]),
          trace_tree_words: StorageArray::new(statics.words[StaticLayout::Tree.get_index()]),
          trace_model_words: StorageArray::new(statics.words[StaticLayout::Model.get_index()]),
          trace_indices: StorageArray::new(statics.indices),
          trace_surfaces: StorageArray::new(statics.surfaces),
          trace_lights: StorageArray::new(bindings.import_buffer(&mut *graph, "light records", &lights.record_buffer)),
          trace_light_view: runtime.push_uniform(&lights.uniform),
          trace_shadow_atlas: bindings.import_view(
            &mut *graph,
            "light shadow atlas",
            scene.lights.get_shadow_atlas().view,
          ),
          trace_shadow_sampler: scene.lights.get_shadow_atlas().sampler,
          trace_sun_maps: lit.shadow_maps,
          trace_sun_shadows: lit.shadows,
          trace_sun_compare: lit.shadow_compare,
          trace_grass_grid: StorageArray::new(grass.grid),
          trace_grass_slots: StorageArray::new(grass.slots),
          trace_grass_models: StorageArray::new(grass.models),
          trace_grass_colours: StorageArray::new(grass.colours),
          trace_grass: grass.uniform,
          trace_grass_cards: StorageArray::new(grass.cards),
          trace_densities: StorageArray::new(statics.trace_densities),
        },
        textures: texture_group,
      })
  }
}

impl ViewFeature for ViewTraceFeature {
  fn release(&mut self) {
    self.trace = None;
  }
}
