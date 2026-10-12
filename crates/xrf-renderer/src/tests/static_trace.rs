use std::sync::Arc;

use glam::{Mat4, Vec3, Vec4};
use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, ProxyAllocator, ProxyHandle};
use xrf_visual::{
  SectorDescription, SectorGeometry, SectorPackage, SectorSection, SectorSurface, VisualClusters, VisualDrawRange,
  VisualSection,
};

use crate::context::gpu_context::GpuContext;
use crate::context::gpu_requirements::can_trace;
use crate::contract::render_backend::RenderBackend;
use crate::frame::frame_encoder::FrameEncoder;
use crate::host::render_asset_source::RenderAssetSource;
use crate::pass::light_record::LightRecord;
use crate::pass::lights_uniform::LightsUniform;
use crate::pass::trace_grass_uniform::TraceGrassUniform;
use crate::pass::trace_parameters::TraceParameters;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_model::StaticModel;
use crate::scene::static_scene::static_model_part::StaticModelPart;
use crate::scene::static_scene::static_model_place::StaticModelPlace;
use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::static_trace::StaticTrace;
use crate::scene::texture::texture_cache::TextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_bundle::checkout_dxc;
use crate::tests::test_workers::create_workers;

/// A source holding no textures.
struct EmptySource;

impl RenderAssetSource for EmptySource {
  fn get_texture_scope(&self) -> String {
    String::from("test")
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(None)
  }
}

/// A ray a probe traces, and what it met: its distance, below none for nothing; its record and triangle; and how far
/// the hit triangle's corners, read back through its record from its arena, interpolated and stood where its instance
/// stands them, are off the ray's point.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Answer {
  distance: f32,
  record: u32,
  primitive: u32,
  error: f32,
}

/// What a probe traces each ray by: the scene's structure, its records, the arenas the hit's corners are read from.
const PROBE: &str = r"
struct Probe {
  origin: vec4<f32>,
  direction: vec4<f32>,
}

struct Answer {
  distance: f32,
  record: u32,
  primitive: u32,
  error: f32,
}

@group(0) @binding(0) var<storage, read> probes: array<Probe>;
@group(0) @binding(1) var<storage, read_write> answers: array<Answer>;

fn probe_word(flags: u32, at: u32) -> u32 {
  if ((flags & 4u) != 0u) {
    return trace_model_words[at];
  }

  return trace_baked_words[at];
}

@compute @workgroup_size(1)
fn cs_probe(@builtin(global_invocation_id) id: vec3<u32>) {
  let probe: Probe = probes[id.x];
  var query: ray_query;

  rayQueryInitialize(&query, trace_scene,
    RayDesc(RAY_FLAG_FORCE_OPAQUE, 0xffu, 0.0, 100.0, probe.origin.xyz, probe.direction.xyz));
  rayQueryProceed(&query);

  let hit: RayIntersection = rayQueryGetCommittedIntersection(&query);

  if (hit.kind == RAY_QUERY_INTERSECTION_NONE) {
    answers[id.x] = Answer(-1.0, 0u, 0u, 0.0);
    return;
  }

  let index: u32 = trace_instances[hit.instance_custom_data].first_record + hit.geometry_index;
  let record: TraceRecord = trace_records[index];
  let weights: vec3<f32> = vec3<f32>(1.0 - hit.barycentrics.x - hit.barycentrics.y, hit.barycentrics);
  var point: vec3<f32> = vec3<f32>(0.0);

  for (var corner: u32 = 0u; corner < 3u; corner++) {
    let at: u32 = (trace_indices[record.first_index + hit.primitive_index * 3u + corner] + record.vertex_start) *
      VERTEX_WORDS;

    point += vertex_position(probe_word(record.flags, at + VERTEX_POSITION),
      probe_word(record.flags, at + VERTEX_POSITION + 1u), probe_word(record.flags, at + VERTEX_POSITION + 2u)) *
      weights[corner];
  }

  answers[id.x] = Answer(hit.t, index, hit.primitive_index,
    distance(hit.object_to_world * vec4<f32>(point, 1.0), probe.origin.xyz + probe.direction.xyz * hit.t));
}
";

/// A sector of two sections: a ground quad two metres across at the origin, and a triangle five metres up ten along.
fn create_sector() -> SectorPackage {
  let positions: [[f32; 3]; 7] = [
    [-1.0, 0.0, -1.0],
    [1.0, 0.0, -1.0],
    [1.0, 0.0, 1.0],
    [-1.0, 0.0, 1.0],
    [9.0, 5.0, -1.0],
    [11.0, 5.0, -1.0],
    [10.0, 5.0, 1.0],
  ];
  let indices: [u32; 9] = [0, 1, 2, 0, 2, 3, 4, 5, 6];
  let mut buffer: Vec<u8> = bytemuck::cast_slice(&positions).to_vec();
  let index_offset: u32 = buffer.len() as u32;

  buffer.extend_from_slice(bytemuck::cast_slice(&indices));

  let end: u32 = buffer.len() as u32;
  let section = |start: u32, count: u32| SectorSection {
    surface: SectorSurface {
      shader_id: 0,
      shader_name: Some(String::from("default")),
      texture_name: Some(String::from("test")),
      hemi: None,
    },
    drawables: Vec::new(),
    draw: VisualDrawRange { start, count },
    bounds: None,
  };

  SectorPackage {
    description: SectorDescription {
      sector: 0,
      geometry: SectorGeometry {
        vertex_count: positions.len() as u32,
        index_count: indices.len() as u32,
        positions: VisualSection {
          byte_offset: 0,
          byte_length: index_offset,
        },
        normals: None,
        tangents: None,
        binormals: None,
        uvs: None,
        uv_components: 0,
        lightmap_uvs: None,
        colors: None,
        indices: VisualSection {
          byte_offset: index_offset,
          byte_length: end - index_offset,
        },
        clusters: VisualClusters {
          ranges: VisualSection {
            byte_offset: end,
            byte_length: 0,
          },
          spheres: VisualSection {
            byte_offset: end,
            byte_length: 0,
          },
        },
      },
      sections: vec![section(0, 6), section(6, 3)],
      instances: Vec::new(),
      impostors: None,
      skipped: Vec::new(),
      bounds: None,
      buffer_length: end,
    },
    buffer,
  }
}

/// A model of one triangle two metres across, flat at its origin, in the model layout: its position in its first three
/// words as floats.
fn create_model() -> StaticModel {
  let positions: [[f32; 3]; 3] = [[-1.0, 0.0, -1.0], [1.0, 0.0, -1.0], [0.0, 0.0, 1.0]];
  let mut words: Vec<u32> = vec![0; StaticLayout::STRIDE as usize * positions.len()];

  for (vertex, position) in positions.iter().enumerate() {
    let at: usize = vertex * StaticLayout::STRIDE as usize + StaticLayout::POSITION;

    words[at..at + 3].copy_from_slice(bytemuck::cast_slice(position));
  }

  StaticModel {
    words,
    indices: vec![0, 1, 2],
    parts: vec![StaticModelPart {
      indices: (0, 3),
      clusters: vec![(0, 1, Vec4::new(0.0, 0.0, 0.0, 1.5))],
      surface: SectorSurface {
        shader_id: 0,
        shader_name: Some(String::from("models/model")),
        texture_name: Some(String::from("test")),
        hemi: None,
      },
      descriptor: None,
    }],
    sphere: Vec4::new(0.0, 0.0, 0.0, 1.5),
    bounds: [Vec3::new(-1.0, 0.0, -1.0), Vec3::new(1.0, 0.0, 1.0)],
    skin: None,
  }
}

/// A headless device that traces rays; none, saying why, where no GPU or no ray query is there.
fn create_tracing_context() -> Option<GpuContext> {
  let Ok(context) = GpuContext::create_headless(RenderBackend::D3d12, checkout_dxc().as_deref())
    .or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan, None))
  else {
    eprintln!("Skipped: no GPU to trace with");

    return None;
  };

  if !can_trace(&context.device) {
    eprintln!("Skipped: '{}' has no ray queries", context.adapter_name);

    return None;
  }

  Some(context)
}

/// A sector's sections become a structure each traced against: a ray meets the ground and the raised triangle at their
/// distances, each on its own record, the hit's corners read back through the record from the arenas landing where the
/// ray met it; a ray past both meets nothing. Skipped, and says so, on a device without ray queries.
#[test]
fn traces_a_sector_through_its_records() {
  let Some(context) = create_tracing_context() else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let source: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let mut textures: TextureCache = TextureCache::new(device, queue, &create_workers());
  let mut scene: StaticScene = StaticScene::new(device, queue);
  let mut sectors: ProxyAllocator<StaticSectorProxy> = ProxyAllocator::new();
  let handle: ProxyHandle<StaticSectorProxy> = sectors.allocate();
  let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));

  scene
    .add_sector(
      device,
      queue,
      &mut encoder,
      &mut textures,
      &source,
      &[],
      (handle, &create_sector()),
    )
    .unwrap();
  scene.prepare_draws(device, queue, &mut encoder);

  let mut trace: StaticTrace = StaticTrace::new(device, queue);

  trace.update(device, queue, &mut encoder, &scene);

  let probes: [[f32; 8]; 3] = [
    [0.5, 10.0, 0.25, 1.0, 0.0, -1.0, 0.0, 0.0],
    [10.0, 10.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0],
    [50.0, 10.0, 50.0, 1.0, 0.0, -1.0, 0.0, 0.0],
  ];
  let answers: Vec<Answer> = run_probes(device, queue, encoder, (&scene, &trace), &probes);

  assert!((answers[0].distance - 10.0).abs() < 1e-3, "{:?}", answers[0]);
  assert!((answers[1].distance - 5.0).abs() < 1e-3, "{:?}", answers[1]);
  assert_eq!(
    (answers[0].record, answers[1].record),
    (0, 1),
    "a record a section, in order"
  );
  assert!(answers[0].error < 1e-3 && answers[1].error < 1e-3, "{answers:?}");
  assert!(answers[2].distance < 0.0, "{:?}", answers[2]);
}

/// Traces each probe's ray against the scene's structure and reads back what it met.
fn run_probes(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  mut encoder: FrameEncoder,
  (scene, trace): (&StaticScene, &StaticTrace),
  probes: &[[f32; 8]],
) -> Vec<Answer> {
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let library: ShaderLibrary = ShaderLibrary::default();
  let source: String =
    library.compose("generated/common/scene_trace").unwrap() + &library.compose("static/vertex").unwrap() + PROBE;
  let module: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("trace probe"),
    source: wgpu::ShaderSource::Wgsl(source.into()),
  });
  let storage = |binding: u32, read_only: bool| wgpu::BindGroupLayoutEntry {
    binding,
    visibility: wgpu::ShaderStages::COMPUTE,
    ty: wgpu::BindingType::Buffer {
      ty: wgpu::BufferBindingType::Storage { read_only },
      has_dynamic_offset: false,
      min_binding_size: None,
    },
    count: None,
  };
  let probe_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
    label: Some("trace probe"),
    entries: &[storage(0, true), storage(1, false)],
  });
  let trace_layout: wgpu::BindGroupLayout = TraceParameters::create_layout(device);
  let pipeline: wgpu::ComputePipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("trace probe"),
    layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("trace probe"),
      bind_group_layouts: &[Some(&probe_layout), None, None, Some(&trace_layout)],
      ..Default::default()
    })),
    module: &module,
    entry_point: Some("cs_probe"),
    compilation_options: Default::default(),
    cache: None,
  });
  let rays: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("trace probe rays"),
    size: size_of_val(probes) as u64,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });
  let answer_bytes: u64 = (probes.len() * size_of::<Answer>()) as u64;
  let answers: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("trace probe answers"),
    size: answer_bytes,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    mapped_at_creation: false,
  });
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("trace probe readback"),
    size: answer_bytes,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });

  queue.write_buffer(&rays, 0, bytemuck::cast_slice(probes));

  let probe_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("trace probe"),
    layout: &probe_layout,
    entries: &[
      wgpu::BindGroupEntry {
        binding: 0,
        resource: rays.as_entire_binding(),
      },
      wgpu::BindGroupEntry {
        binding: 1,
        resource: answers.as_entire_binding(),
      },
    ],
  });
  // `TraceParameters`' buffers in their order, after the structure.
  let buffers: [&wgpu::Buffer; 8] = [
    trace.get_instances(),
    trace.get_records(),
    scene.geometry.places.get_buffer(),
    scene.geometry.words[StaticLayout::Baked.get_index()].get_buffer(),
    scene.geometry.words[StaticLayout::Tree.get_index()].get_buffer(),
    scene.geometry.words[StaticLayout::Model.get_index()].get_buffer(),
    scene.geometry.indices.get_buffer(),
    scene.surfaces.buffer.get_buffer(),
  ];
  let mut entries: Vec<wgpu::BindGroupEntry<'_>> = vec![wgpu::BindGroupEntry {
    binding: 0,
    resource: trace.get_structure().tlas.as_binding(),
  }];

  entries.extend(buffers.iter().enumerate().map(|(index, buffer)| wgpu::BindGroupEntry {
    binding: index as u32 + 1,
    resource: buffer.as_entire_binding(),
  }));

  // No light, and an atlas of one texel: the probe lights nothing.
  let lights: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("trace probe lights"),
    size: size_of::<LightRecord>() as u64,
    usage: wgpu::BufferUsages::STORAGE,
    mapped_at_creation: false,
  });
  let light_view: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("trace probe light view"),
    size: size_of::<LightsUniform>() as u64,
    usage: wgpu::BufferUsages::UNIFORM,
    mapped_at_creation: false,
  });
  let atlas: wgpu::TextureView = device
    .create_texture(&wgpu::TextureDescriptor {
      label: Some("trace probe atlas"),
      size: wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Depth32Float,
      usage: wgpu::TextureUsages::TEXTURE_BINDING,
      view_formats: &[],
    })
    .create_view(&Default::default());
  let first_light: u32 = buffers.len() as u32 + 1;

  entries.extend([
    wgpu::BindGroupEntry {
      binding: first_light,
      resource: lights.as_entire_binding(),
    },
    wgpu::BindGroupEntry {
      binding: first_light + 1,
      resource: light_view.as_entire_binding(),
    },
    wgpu::BindGroupEntry {
      binding: first_light + 2,
      resource: wgpu::BindingResource::TextureView(&atlas),
    },
  ]);

  // No grass either: its grid, slots, models and colours nothing, its uniform naming no model.
  let grass: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("trace probe grass"),
    size: size_of::<TraceGrassUniform>() as u64,
    usage: wgpu::BufferUsages::UNIFORM,
    mapped_at_creation: false,
  });

  entries.extend([
    wgpu::BindGroupEntry {
      binding: first_light + 3,
      resource: trace.get_empty().as_entire_binding(),
    },
    wgpu::BindGroupEntry {
      binding: first_light + 4,
      resource: trace.get_empty().as_entire_binding(),
    },
    wgpu::BindGroupEntry {
      binding: first_light + 5,
      resource: trace.get_empty().as_entire_binding(),
    },
    wgpu::BindGroupEntry {
      binding: first_light + 6,
      resource: trace.get_empty().as_entire_binding(),
    },
    wgpu::BindGroupEntry {
      binding: first_light + 7,
      resource: grass.as_entire_binding(),
    },
  ]);

  let trace_group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("trace probe scene"),
    layout: &trace_layout,
    entries: &entries,
  });

  {
    let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&Default::default());

    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &probe_group, &[]);
    pass.set_bind_group(3, &trace_group, &[0, 0]);
    pass.dispatch_workgroups(probes.len() as u32, 1, 1);
  }

  encoder.copy_buffer_to_buffer(&answers, 0, &readback, 0, answer_bytes);
  queue.submit([encoder.finish().0]);
  readback.slice(..).map_async(wgpu::MapMode::Read, |_| ());
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  let bytes: Vec<u8> = readback.slice(..).get_mapped_range().unwrap().to_vec();

  if let Some(error) = pollster::block_on(scope.pop()) {
    panic!("The probe does not validate: {error}");
  }

  bytemuck::cast_slice(&bytes).to_vec()
}

/// A spawned model is traced where each of its objects stands, both on the model's one record, its corners landing
/// where the ray met it; an object taken out of the scene is traced no more.
#[test]
fn traces_spawned_objects_where_they_stand() {
  let Some(context) = create_tracing_context() else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let source: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let mut textures: TextureCache = TextureCache::new(device, queue, &create_workers());
  let mut scene: StaticScene = StaticScene::new(device, queue);
  let mut sectors: ProxyAllocator<StaticSectorProxy> = ProxyAllocator::new();
  let mut models: ProxyAllocator<StaticModelProxy> = ProxyAllocator::new();
  let mut objects: ProxyAllocator<StaticObjectProxy> = ProxyAllocator::new();
  let model: ProxyHandle<StaticModelProxy> = models.allocate();
  let standing: [ProxyHandle<StaticObjectProxy>; 2] = [objects.allocate(), objects.allocate()];
  let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));

  scene
    .add_sector(
      device,
      queue,
      &mut encoder,
      &mut textures,
      &source,
      &[],
      (sectors.allocate(), &create_sector()),
    )
    .unwrap();
  scene
    .add_model(
      device,
      queue,
      &mut encoder,
      (&mut textures, &source),
      (model, &create_model()),
    )
    .unwrap();

  for (index, handle) in standing.iter().enumerate() {
    let place: StaticModelPlace = StaticModelPlace {
      object: index as u32,
      transform: Mat4::from_translation(Vec3::new(30.0 + index as f32 * 10.0, 2.0 + index as f32, 0.0)),
      lighting: None,
      group: 0,
    };

    scene.add_object(device, queue, *handle, (model, &place)).unwrap();
  }

  scene.prepare_draws(device, queue, &mut encoder);

  let mut trace: StaticTrace = StaticTrace::new(device, queue);

  trace.update(device, queue, &mut encoder, &scene);

  let probes: [[f32; 8]; 2] = [
    [30.0, 10.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0],
    [40.0, 10.0, 0.0, 1.0, 0.0, -1.0, 0.0, 0.0],
  ];
  let answers: Vec<Answer> = run_probes(device, queue, encoder, (&scene, &trace), &probes);

  assert!((answers[0].distance - 8.0).abs() < 1e-3, "{:?}", answers[0]);
  assert!((answers[1].distance - 7.0).abs() < 1e-3, "{:?}", answers[1]);
  assert_eq!(
    (answers[0].record, answers[1].record),
    (2, 2),
    "the model's record, after the sector's two"
  );
  assert!(answers[0].error < 1e-3 && answers[1].error < 1e-3, "{answers:?}");

  let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));

  scene.remove_object(standing[0]);
  scene.prepare_draws(device, queue, &mut encoder);
  trace.update(device, queue, &mut encoder, &scene);

  let answers: Vec<Answer> = run_probes(device, queue, encoder, (&scene, &trace), &probes);

  assert!(answers[0].distance < 0.0, "{:?}", answers[0]);
  assert!((answers[1].distance - 7.0).abs() < 1e-3, "{:?}", answers[1]);
}
