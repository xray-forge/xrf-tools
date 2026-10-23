use glam::{Vec3, Vec4};

use crate::lighting::light_specular::to_light_specular;
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

// At the authored levels the lights' specular joins the diffuse in display values, as the engine adds them before its
// display decodes the sum: the engine's specular weight over its colour, tinted by the lights' colour where Anomaly
// tints it, both back in the engine's units. A dim moon's broad lobe over a dark wet surface then adds what the
// engine's display shows of it, a sliver of what it added as light.
#[test]
fn the_authored_specular_joins_the_diffuse_as_the_engine_adds_it() {
  let (scale, gain): (f32, f32) = (64.0, 2.0);
  let unit: f32 = scale * gain;
  let moon: Vec3 = Vec3::splat(0.1);
  let gloss: f32 = 0.8;
  let glossed: f32 = gloss * to_light_specular(moon, false) * unit;
  let shade: f32 = 0.05;
  let lit: Vec3 = Vec3::splat(srgb_decoded(shade) * scale);
  let light: Vec3 = authored_light(moon, true) * unit;
  let body = |is_tinted: bool| {
    format!("answer = summed_specular(first.rgb, second.rgb, first.w, {scale:?}, second.w, {is_tinted});")
  };
  let cases: [(Vec4, Vec4); 1] = [(lit.extend(glossed), light.extend(gain))];
  let (Some(tinted), Some(white)) = (run(&body(true), &cases), run(&body(false), &cases)) else {
    return;
  };
  let joined = |added: Vec3| encoded_sum(shade, added) * scale;

  assert_close(tinted[0], joined(moon * glossed / unit), "tinted");
  assert_close(white[0], joined(Vec3::splat(glossed / unit)), "white");
  assert!(
    (tinted[0] - lit).max_element() * 50.0 < glossed,
    "the moon's specular {} against {glossed} added as light",
    (tinted[0] - lit).max_element()
  );
}

/// The light a display value `shade` shows with `added` summed onto it in display values.
fn encoded_sum(shade: f32, added: Vec3) -> Vec3 {
  decoded(Vec3::splat(shade) + added)
}

/// What the engine's meter reads off a texel of its eight-bit high part holding `high`: twice it, held to one, by
/// `LUMINANCE_VECTOR`, over `def_hdr`.
fn engine_reading(high: Vec3) -> f32 {
  let held: Vec3 = (high.clamp(Vec3::ZERO, Vec3::ONE) * 255.0).round() / 255.0;

  (held * 2.0).min(Vec3::ONE).dot(Vec3::new(0.3, 0.38, 0.22)) * 9.0
}

// The authored levels meter as the engine does: a texel of their scene, the engine's display values decoded at the
// engine-equivalent scale, encoded reads as the engine's meter reads that colour exposed by the scale, saturating as
// its eight bits do; the sky, as the display shows it, reads as the engine's high part holds the sky as drawn.
#[test]
fn the_authored_scene_is_metered_as_the_engines_high_part() {
  let colour: Vec3 = Vec3::new(0.4, 0.3, 0.2);
  let picture: Vec3 = Vec3::new(0.6, 0.7, 0.85);
  let cases: [(Vec4, Vec4); 3] = [
    (colour.extend(1.3), Vec4::ZERO),
    (colour.extend(16.0), Vec4::ZERO),
    (picture.extend(1.0), Vec4::ONE),
  ];
  let Some(read) = run(
    "let land: vec3<f32> = srgb_encoded(srgb_decoded(first.rgb * first.w)); \
     answer = vec3<f32>(engine_metered_shown(select(land, first.rgb, second.w > 0.5)));",
    &cases,
  ) else {
    return;
  };

  for (shown, expected, case) in [
    (read[0].x, engine_reading(colour * 1.3 / 9.0), "land"),
    (read[1].x, engine_reading(colour * 16.0 / 9.0), "bright land"),
    (read[2].x, engine_reading(picture / 9.0), "sky"),
  ] {
    assert!((shown - expected).abs() < 1e-3, "{case}: {shown} against {expected}");
  }
}
