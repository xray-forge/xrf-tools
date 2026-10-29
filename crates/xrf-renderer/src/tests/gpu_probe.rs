//! Runs a shader module's functions on the GPU over cases of two vectors each, for tests holding WGSL to Rust
//! references or to what it must integrate to.

use glam::{Vec3, Vec4};

use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_context::create_test_context;

/// A compute entry over the cases: each case's two vectors as `first` and `second`, `body` setting `answer`, written
/// back over the first.
fn probe(body: &str) -> String {
  format!(
    r#"
struct ProbeCase {{
  first: vec4<f32>,
  second: vec4<f32>,
}}

@group(0) @binding(0) var<storage, read_write> cases: array<ProbeCase>;

@compute @workgroup_size(1)
fn cs_probe(@builtin(global_invocation_id) id: vec3<u32>) {{
  let first: vec4<f32> = cases[id.x].first;
  let second: vec4<f32> = cases[id.x].second;
  var answer: vec3<f32>;

  {body}

  cases[id.x].first = vec4<f32>(answer, 0.0);
}}
"#
  )
}

/// Runs `body` over each case after `module`'s functions (`probe`), answering each case's `answer`; none without a GPU
/// to run it on (`create_test_context`). Panics where the probe does not validate.
pub fn run_gpu_probe(module: &str, body: &str, cases: &[(Vec4, Vec4)]) -> Option<Vec<Vec3>> {
  let context = create_test_context(&format!("to run {module}'s functions"))?;
  let (device, queue) = (&context.device, &context.queue);
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let source: String = ShaderLibrary::default().compose(module).unwrap() + &probe(body);
  let shader: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("gpu probe"),
    source: wgpu::ShaderSource::Wgsl(source.into()),
  });
  let pipeline: wgpu::ComputePipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("gpu probe"),
    layout: None,
    module: &shader,
    entry_point: Some("cs_probe"),
    compilation_options: Default::default(),
    cache: None,
  });
  let data: Vec<Vec4> = cases.iter().flat_map(|(first, second)| [*first, *second]).collect();
  let bytes: u64 = size_of_val(data.as_slice()) as u64;
  let buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("gpu probe cases"),
    size: bytes,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    mapped_at_creation: false,
  });
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("gpu probe readback"),
    size: bytes,
    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });

  queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&data));

  let group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("gpu probe"),
    layout: &pipeline.get_bind_group_layout(0),
    entries: &[wgpu::BindGroupEntry {
      binding: 0,
      resource: buffer.as_entire_binding(),
    }],
  });
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

  {
    let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&Default::default());

    pass.set_pipeline(&pipeline);
    pass.set_bind_group(0, &group, &[]);
    pass.dispatch_workgroups(cases.len() as u32, 1, 1);
  }

  encoder.copy_buffer_to_buffer(&buffer, 0, &readback, 0, bytes);
  queue.submit([encoder.finish()]);
  readback.slice(..).map_async(wgpu::MapMode::Read, |_| ());
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  let answers: Vec<Vec4> = bytemuck::cast_slice(&readback.slice(..).get_mapped_range().unwrap()).to_vec();

  if let Some(error) = pollster::block_on(scope.pop()) {
    panic!("The {module} probe does not validate: {error}");
  }

  Some(answers.chunks(2).map(|case| case[0].truncate()).collect())
}
