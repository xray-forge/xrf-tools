use glam::Vec4;

use crate::param::tests::fixtures::{Blur, Scale, Settings, Shade};
use crate::{
  BindGroupCache, FrameGraph, GraphBindings, GraphBufferAccess, GraphBufferDescriptor, GraphCompileOptions,
  GraphTextureAccess, GraphTextureDescriptor, PassParameters, ShaderDeclarations, StorageArray, StorageArrayMut,
  TransientPool, UniformBinding, UploadRing,
};

#[test]
fn derives_the_layout_with_writable_bindings_kept_from_the_vertex_stage() {
  let entries: Vec<wgpu::BindGroupLayoutEntry> = Blur::get_layout_entries();
  let every: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT | wgpu::ShaderStages::COMPUTE;

  assert_eq!(entries.len(), 4);
  assert_eq!(
    entries.iter().map(|entry| entry.binding).collect::<Vec<_>>(),
    [0, 1, 2, 3]
  );
  assert_eq!(entries[0].visibility, every);
  assert!(matches!(
    entries[0].ty,
    wgpu::BindingType::Texture {
      sample_type: wgpu::TextureSampleType::Float { filterable: true },
      view_dimension: wgpu::TextureViewDimension::D2,
      ..
    }
  ));
  assert!(matches!(
    entries[1].ty,
    wgpu::BindingType::Texture {
      sample_type: wgpu::TextureSampleType::Depth,
      ..
    }
  ));
  assert_eq!(
    entries[2].visibility,
    wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE
  );
  assert!(matches!(
    entries[2].ty,
    wgpu::BindingType::StorageTexture {
      access: wgpu::StorageTextureAccess::WriteOnly,
      format: wgpu::TextureFormat::Rgba16Float,
      ..
    }
  ));
  assert!(matches!(
    entries[3].ty,
    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
  ));
}

#[test]
fn derives_the_wgsl_bindings_named_as_the_fields() {
  assert_eq!(
    Blur::get_wgsl_bindings(),
    "@group(1) @binding(0) var source: texture_2d<f32>;\n\
     @group(1) @binding(1) var depth: texture_depth_2d;\n\
     @group(1) @binding(2) var destination: texture_storage_2d<rgba16float, write>;\n\
     @group(1) @binding(3) var linear: sampler;\n"
  );
  assert_eq!(
    Scale::get_wgsl_bindings(),
    "@group(0) @binding(0) var<uniform> settings: Settings;\n\
     @group(0) @binding(1) var<storage, read> input: array<u32>;\n\
     @group(0) @binding(2) var<storage, read_write> output: array<u32>;\n"
  );
}

#[test]
fn declares_the_accesses_its_bindings_make() {
  let mut graph: FrameGraph<'_> = FrameGraph::new();
  let source = graph.create_texture(GraphTextureDescriptor::new_2d(
    "source",
    8,
    8,
    wgpu::TextureFormat::Rgba8Unorm,
  ));
  let depth = graph.create_texture(GraphTextureDescriptor::new_2d(
    "depth",
    8,
    8,
    wgpu::TextureFormat::Depth32Float,
  ));
  let destination = graph.create_texture(GraphTextureDescriptor::new_2d(
    "destination",
    8,
    8,
    wgpu::TextureFormat::Rgba16Float,
  ));
  let input = graph.create_buffer(GraphBufferDescriptor::new("input", 64));
  let output = graph.create_buffer(GraphBufferDescriptor::new("output", 64));
  let shade: Shade = Shade {
    source,
    depth,
    destination,
  };

  assert_eq!(
    shade.list_texture_accesses(),
    [
      (source, GraphTextureAccess::Sampled),
      (depth, GraphTextureAccess::Sampled),
      (destination, GraphTextureAccess::StorageWrite)
    ]
  );

  let storage = (StorageArray::<u32>::new(input), StorageArrayMut::<u32>::new(output));

  assert_eq!(
    crate::StorageField::get_access(&storage.0),
    GraphBufferAccess::StorageRead
  );
  assert_eq!(
    crate::StorageField::get_access(&storage.1),
    GraphBufferAccess::StorageReadWrite
  );
}

#[test]
fn derives_bindings_naga_accepts_in_a_shader() {
  let mut declarations: ShaderDeclarations = ShaderDeclarations::new();

  Scale::declare(&mut declarations);

  let source: String = format!(
    "{}\n{}\n{}\n{}",
    declarations.to_wgsl(),
    Scale::get_wgsl_bindings(),
    Blur::get_wgsl_bindings(),
    "@compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id: vec3<u32>) {
       output[id.x] = input[id.x] * u32(settings.factor.x);
       let color: vec4<f32> = textureSampleLevel(source, linear, vec2<f32>(0.5), 0.0);
       let near: f32 = textureLoad(depth, vec2<i32>(0), 0);
       textureStore(destination, vec2<i32>(0), color * near);
     }"
  );
  let module: naga::Module =
    naga::front::wgsl::parse_str(&source).unwrap_or_else(|error| panic!("{}", error.emit_to_string(&source)));

  naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::all())
    .validate(&module)
    .unwrap();
}

#[test]
fn binds_parameters_in_a_compute_pass_and_reuses_the_bind_group() {
  let Some((device, queue)) = crate::tests::test_device::create_device() else {
    return;
  };
  let size: u64 = 64 * 4;
  let buffer = |label: &'static str, usage: wgpu::BufferUsages| {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(label),
      size,
      usage,
      mapped_at_creation: false,
    })
  };
  let input_buffer: wgpu::Buffer = buffer("input", wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST);
  let output_buffer: wgpu::Buffer = buffer("output", wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC);
  let readback: wgpu::Buffer = buffer("readback", wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ);

  queue.write_buffer(
    &input_buffer,
    0,
    bytemuck::cast_slice(&(0..64u32).collect::<Vec<u32>>()),
  );

  let mut ring: UploadRing = UploadRing::new(&device, "uniforms", wgpu::BufferUsages::UNIFORM, 256);
  let cache: BindGroupCache = BindGroupCache::new();
  let mut pool: TransientPool = TransientPool::new();
  let mut declarations: ShaderDeclarations = ShaderDeclarations::new();

  Scale::declare(&mut declarations);

  let module: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("scale"),
    source: wgpu::ShaderSource::Wgsl(
      format!(
        "{}\n{}\n@compute @workgroup_size(64) fn main(@builtin(global_invocation_id) id: vec3<u32>) {{
           output[id.x] = input[id.x] * u32(settings.factor.x);
         }}",
        declarations.to_wgsl(),
        Scale::get_wgsl_bindings()
      )
      .into(),
    ),
  });
  let layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("scale"),
    bind_group_layouts: &[Some(&cache.get_layout::<Scale<'_>>(&device))],
    ..Default::default()
  });
  let pipeline: wgpu::ComputePipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("scale"),
    layout: Some(&layout),
    module: &module,
    entry_point: Some("main"),
    compilation_options: Default::default(),
    cache: None,
  });

  for factor in [2.0f32, 3.0] {
    let slice = ring.push(&Settings {
      factor: Vec4::splat(factor),
    });

    ring.flush(&device, &queue);

    let mut graph: FrameGraph<'_> = FrameGraph::new();
    let input = graph.import_buffer(GraphBufferDescriptor::new("input", size));
    let output = graph.import_buffer(GraphBufferDescriptor::new("output", size));
    let scale: Scale<'_> = Scale {
      settings: UniformBinding::new(ring.get_buffer(), slice),
      input: StorageArray::new(input),
      output: StorageArrayMut::new(output),
    };
    let pipeline: &wgpu::ComputePipeline = &pipeline;

    graph
      .add_compute_pass("scale")
      .parameters(&scale)
      .record(move |context| {
        context.bind(&scale);
        context.get_pass().set_pipeline(pipeline);
        context.get_pass().dispatch_workgroups(1, 1, 1);
      });

    let compiled = graph.compile(&GraphCompileOptions::default()).unwrap();
    let mut bindings: GraphBindings<'_> = GraphBindings::new();

    bindings
      .bind_buffer(input, &input_buffer)
      .bind_buffer(output, &output_buffer);
    queue.submit(compiled.execute(&device, &mut pool, &cache, &bindings).unwrap());
  }

  assert_eq!(
    cache.get_bind_group_count(),
    1,
    "the second frame binds the same resources"
  );

  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

  encoder.copy_buffer_to_buffer(&output_buffer, 0, &readback, 0, size);
  queue.submit([encoder.finish()]);
  readback.slice(..).map_async(wgpu::MapMode::Read, |_| {});
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  let words: Vec<u32> = bytemuck::cast_slice(&readback.slice(..).get_mapped_range().unwrap()).to_vec();

  assert_eq!(
    words[..4],
    [0, 3, 6, 9],
    "the second frame's factor, read at its own offset"
  );
  assert_eq!(words[63], 189);
}
