use glam::{Vec3, Vec4};

use crate::lighting::srgb::{authored_light, srgb_decoded};
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_context::create_test_context;

/// Runs `body` over each case of `common/hmodel`'s functions: a case is two vectors, `first` and `second`, and the
/// body sets `answer`.
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

/// What the GPU answers each case with under `body`.
fn run(body: &str, cases: &[(Vec4, Vec4)]) -> Option<Vec<Vec3>> {
  let context = create_test_context("to run the authored light's functions")?;
  let (device, queue) = (&context.device, &context.queue);
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let source: String = ShaderLibrary::default().compose("common/hmodel").unwrap() + &probe(body);
  let module: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("authored light probe"),
    source: wgpu::ShaderSource::Wgsl(source.into()),
  });
  let pipeline: wgpu::ComputePipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("authored light probe"),
    layout: None,
    module: &module,
    entry_point: Some("cs_probe"),
    compilation_options: Default::default(),
    cache: None,
  });
  let data: Vec<Vec4> = cases.iter().flat_map(|(first, second)| [*first, *second]).collect();
  let bytes: u64 = size_of_val(data.as_slice()) as u64;
  let buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("authored light probe cases"),
    size: bytes,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    mapped_at_creation: false,
  });
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("authored light probe readback"),
    size: bytes,
    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });

  queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&data));

  let group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("authored light probe"),
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

  let shown: Vec<Vec4> = bytemuck::cast_slice(&readback.slice(..).get_mapped_range().unwrap()).to_vec();

  if let Some(error) = pollster::block_on(scope.pop()) {
    panic!("The authored light probe does not validate: {error}");
  }

  Some(shown.chunks(2).map(|case| case[0].truncate()).collect())
}

fn assert_close(actual: Vec3, expected: Vec3, case: &str) {
  assert!(
    (actual - expected).abs().max_element() <= expected.abs().max_element() * 1e-4 + 1e-7,
    "{case}: {actual} against {expected}"
  );
}

fn decoded(value: Vec3) -> Vec3 {
  Vec3::new(srgb_decoded(value.x), srgb_decoded(value.y), srgb_decoded(value.z))
}

// At the authored levels (a gain) the hemisphere is the engine's own square in the engine's units, its hue decoded per
// channel: its luminance the square's, its channels standing at the fourth-and-more power of the engine's colour, as
// the display decodes the square; at the physical levels (no gain) the cubes decoded and the colour's hue.
#[test]
fn the_authored_hemisphere_is_the_engines_square_decoded_per_channel() {
  let colour: Vec3 = Vec3::new(0.6, 0.5, 0.35);
  let irradiance: Vec3 = Vec3::new(0.7, 0.8, 0.9);
  let gain: f32 = 1.87;
  let Some(shown) = run(
    "answer = physical_hemisphere(first.rgb, second.rgb, first.w);",
    &[
      (colour.extend(gain), irradiance.extend(0.0)),
      (colour.extend(0.0), irradiance.extend(0.0)),
    ],
  ) else {
    return;
  };
  let engine: Vec3 = colour * irradiance / gain;

  assert_close(shown[0], authored_light(engine * engine, true) * gain, "authored");
  assert_close(
    shown[0] / shown[0].x,
    decoded(engine * engine / (engine.x * engine.x)),
    "authored hue",
  );
  assert_close(shown[1], authored_light(colour, true) * decoded(irradiance), "physical");
}

// A light's authored display value comes back from the light `srgb_light` made of it, whatever its strength, as the
// froxels take a lamp's colour back to gather its shafts as the engine does.
#[test]
fn srgb_light_authored_undoes_srgb_light() {
  let colours: [Vec3; 3] = [
    Vec3::new(1.0, 0.6, 0.3),
    Vec3::new(0.2, 0.25, 0.4) * 3.0,
    Vec3::new(0.5, 0.5, 0.5),
  ];
  let cases: Vec<(Vec4, Vec4)> = colours.iter().map(|colour| (colour.extend(0.0), Vec4::ZERO)).collect();
  let Some(shown) = run("answer = srgb_light_authored(srgb_light(first.rgb));", &cases) else {
    return;
  };

  for (shown, colour) in shown.iter().zip(colours) {
    assert_close(*shown, colour, &format!("{colour}"));
  }
}
