//! Runs an expression of a shader module on the GPU over cases of two vectors each, which is how a test holds a
//! module's functions to their definitions.

use glam::Vec4;

use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_context::create_test_context;

/// Each case's two vectors in as `probe.first` and `probe.second`, `ANSWER` out.
const PROBE: &str = r"
struct ProbeCase {
  first: vec4<f32>,
  second: vec4<f32>,
};

@group(0) @binding(0) var<storage, read_write> cases: array<ProbeCase>;

@compute @workgroup_size(1)
fn cs_probe(@builtin(global_invocation_id) id: vec3<u32>) {
  let probe: ProbeCase = cases[id.x];

  cases[id.x].first = ANSWER;
}
";

/// What the GPU answers each case with under `answer`, a `vec4<f32>` expression of `module`'s functions over the case's
/// `probe.first` and `probe.second`; `None` where no device can run it.
///
/// # Panics
///
/// Panics where the probe does not validate.
pub(crate) fn run_wgsl_probe(module: &str, answer: &str, cases: &[(Vec4, Vec4)]) -> Option<Vec<Vec4>> {
  let context = create_test_context(&format!("to run `{module}`"))?;
  let (device, queue) = (&context.device, &context.queue);
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let source: String = ShaderLibrary::default().compose(module).unwrap() + &PROBE.replace("ANSWER", answer);
  let shader: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("wgsl probe"),
    source: wgpu::ShaderSource::Wgsl(source.into()),
  });
  let pipeline: wgpu::ComputePipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("wgsl probe"),
    layout: None,
    module: &shader,
    entry_point: Some("cs_probe"),
    compilation_options: Default::default(),
    cache: None,
  });
  let data: Vec<Vec4> = cases.iter().flat_map(|(first, second)| [*first, *second]).collect();
  let bytes: u64 = size_of_val(data.as_slice()) as u64;
  let buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("wgsl probe cases"),
    size: bytes,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    mapped_at_creation: false,
  });
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("wgsl probe readback"),
    size: bytes,
    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });

  queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&data));

  let group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("wgsl probe"),
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

  let answered: Vec<Vec4> = bytemuck::cast_slice(&readback.slice(..).get_mapped_range().unwrap()).to_vec();

  if let Some(error) = pollster::block_on(scope.pop()) {
    panic!("The probe of `{module}` does not validate: {error}");
  }

  Some(answered.chunks(2).map(|case| case[0]).collect())
}
