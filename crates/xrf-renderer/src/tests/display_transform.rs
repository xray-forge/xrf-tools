//! The physical light's display transforms and grading run on the GPU as the present runs them, held to their published
//! definitions transcribed here: Khronos PBR Neutral's reference, three.js's AgX, and the engine's curve.

use glam::{Mat3, Vec3, Vec4};

use crate::lighting::srgb::srgb_encoded;
use crate::pass::display_mode::DisplayMode;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_context::create_test_context;

/// What the probe shows each case as: `shown_for_display` over its colour under its display.
const PROBE: &str = r"
struct DisplayCase {
  colour: vec4<f32>,
  display: vec4<f32>,
};

@group(0) @binding(0) var<storage, read_write> cases: array<DisplayCase>;

@compute @workgroup_size(1)
fn cs_display(@builtin(global_invocation_id) id: vec3<u32>) {
  let probe: DisplayCase = cases[id.x];

  cases[id.x].colour = vec4<f32>(shown_for_display(probe.colour.rgb, probe.display), 1.0);
}
";

/// The engine's curve's white, as `tonemap` squares it.
const WHITE: f32 = 1.7;

/// The engine's curve a channel, `x (1 + x / W²) / (1 + x)`.
fn engine_curve(light: Vec3) -> Vec3 {
  light * (Vec3::ONE + light / (WHITE * WHITE)) / (Vec3::ONE + light)
}

/// Khronos PBR Neutral, as the reference GLSL of the Khronos PBR Neutral specification writes it.
fn pbr_neutral(colour: Vec3) -> Vec3 {
  let start: f32 = 0.8 - 0.04;
  let desaturation: f32 = 0.15;
  let least: f32 = colour.min_element();
  let offset: f32 = if least < 0.08 {
    least - 6.25 * least * least
  } else {
    0.04
  };
  let toed: Vec3 = colour - Vec3::splat(offset);
  let peak: f32 = toed.max_element();

  if peak < start {
    return toed.clamp(Vec3::ZERO, Vec3::ONE);
  }

  let d: f32 = 1.0 - start;
  let compressed: f32 = 1.0 - d * d / (peak + d - start);
  let grey: f32 = 1.0 - 1.0 / (desaturation * (peak - compressed) + 1.0);

  (toed * (compressed / peak))
    .lerp(Vec3::splat(compressed), grey)
    .clamp(Vec3::ZERO, Vec3::ONE)
}

/// AgX's base look as three.js's `AgXToneMapping` writes it: Rec. 2020, the inset, a log encoding over −12.47 to +4.03
/// stops, the polynomial sigmoid, the outset, the 2.2 power, back to sRGB.
fn agx(colour: Vec3) -> Vec3 {
  let to_rec2020: Mat3 = Mat3::from_cols(
    Vec3::new(0.6274, 0.0691, 0.0164),
    Vec3::new(0.3293, 0.9195, 0.0880),
    Vec3::new(0.0433, 0.0113, 0.8956),
  );
  let from_rec2020: Mat3 = Mat3::from_cols(
    Vec3::new(1.6605, -0.1246, -0.0182),
    Vec3::new(-0.5876, 1.1329, -0.1006),
    Vec3::new(-0.0728, -0.0083, 1.1187),
  );
  let inset: Mat3 = Mat3::from_cols(
    Vec3::new(0.85662715, 0.13731897, 0.11189821),
    Vec3::new(0.09512124, 0.761242, 0.07679942),
    Vec3::new(0.048251606, 0.10143904, 0.81130237),
  );
  let outset: Mat3 = Mat3::from_cols(
    Vec3::new(1.1271006, -0.14132976, -0.14132976),
    Vec3::new(-0.11060664, 1.1578237, -0.11060664),
    Vec3::new(-0.016493939, -0.016493939, 1.2519364),
  );
  let (least, most): (f32, f32) = (-12.47393, 4.026069);
  let encoded: Vec3 = (inset * (to_rec2020 * colour))
    .max(Vec3::splat(1e-10))
    .to_array()
    .map(|channel| ((channel.log2() - least) / (most - least)).clamp(0.0, 1.0))
    .into();
  let sigmoid = |x: f32| {
    let x2: f32 = x * x;
    let x4: f32 = x2 * x2;

    15.5 * x4 * x2 - 40.14 * x4 * x + 31.96 * x4 - 6.868 * x2 * x + 0.4298 * x2 + 0.1191 * x - 0.00232
  };
  let curved: Vec3 = encoded.to_array().map(sigmoid).into();
  let shown: Vec3 = (outset * curved).max(Vec3::ZERO).powf(2.2);

  (from_rec2020 * shown).clamp(Vec3::ZERO, Vec3::ONE)
}

/// The grading before a physical transform: luminance to `contrast` in stops about middle grey, colour mixed from it by
/// `saturation`.
fn graded(light: Vec3, contrast: f32, saturation: f32) -> Vec3 {
  let luminance_was: f32 = light.dot(Vec3::new(0.2126, 0.7152, 0.0722)).max(1e-6);
  let luminance_is: f32 = 0.18 * (luminance_was / 0.18).powf(contrast);

  Vec3::splat(luminance_is)
    .lerp(light * (luminance_is / luminance_was), saturation)
    .max(Vec3::ZERO)
}

fn encoded(colour: Vec3) -> Vec3 {
  colour.to_array().map(srgb_encoded).into()
}

/// A transform as its definition writes it, light to the shown linear colour.
type Reference = fn(Vec3) -> Vec3;

/// What the GPU shows each colour as under each display.
fn show(cases: &[(Vec3, Vec4)]) -> Option<Vec<Vec3>> {
  let context = create_test_context("to run the display transforms")?;
  let (device, queue) = (&context.device, &context.queue);
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let library: ShaderLibrary = ShaderLibrary::default();
  let source: String = library.compose("common/display_transform").unwrap() + PROBE;
  let module: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("display probe"),
    source: wgpu::ShaderSource::Wgsl(source.into()),
  });
  let pipeline: wgpu::ComputePipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
    label: Some("display probe"),
    layout: None,
    module: &module,
    entry_point: Some("cs_display"),
    compilation_options: Default::default(),
    cache: None,
  });
  let data: Vec<Vec4> = cases
    .iter()
    .flat_map(|(colour, display)| [colour.extend(1.0), *display])
    .collect();
  let bytes: u64 = size_of_val(data.as_slice()) as u64;
  let buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("display probe cases"),
    size: bytes,
    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
    mapped_at_creation: false,
  });
  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("display probe readback"),
    size: bytes,
    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });

  queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&data));

  let group: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("display probe"),
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
    panic!("The display probe does not validate: {error}");
  }

  Some(shown.chunks(2).map(|case| case[0].truncate()).collect())
}

fn display(mode: DisplayMode, contrast: f32, saturation: f32) -> Vec4 {
  Vec4::new(mode.get_index() as f32, contrast, saturation, 0.0)
}

/// Light from the black end to well past white, greys and saturated colours, and the chart's orange.
fn colours() -> Vec<Vec3> {
  vec![
    Vec3::ZERO,
    Vec3::splat(0.01),
    Vec3::splat(0.18),
    Vec3::splat(0.5),
    Vec3::splat(1.0),
    Vec3::splat(4.0),
    Vec3::splat(16.0),
    Vec3::new(0.5, 0.2, 0.05),
    Vec3::new(4.0, 1.0, 0.1),
    Vec3::new(0.05, 0.3, 0.8),
    Vec3::new(0.41, 0.21, 0.03),
  ]
}

fn assert_close(actual: Vec3, expected: Vec3, what: &str) {
  assert!(
    (actual - expected).abs().max_element() < 2e-3,
    "{what}: shown {actual}, defined {expected}"
  );
}

/// Each physical transform and the engine's curve show light as their definitions do, ungraded.
#[test]
fn the_transforms_show_light_as_their_definitions_do() {
  let modes: [(DisplayMode, Reference); 3] = [
    (DisplayMode::PbrNeutral, pbr_neutral),
    (DisplayMode::Agx, agx),
    (DisplayMode::EngineEncoded, engine_curve),
  ];
  let cases: Vec<(Vec3, Vec4)> = modes
    .iter()
    .flat_map(|(mode, _)| colours().into_iter().map(|colour| (colour, display(*mode, 1.0, 1.0))))
    .collect();
  let Some(shown) = show(&cases) else {
    return;
  };

  for (index, (colour, _)) in cases.iter().enumerate() {
    let (mode, defined) = modes[index / colours().len()];
    let expected: Vec3 = encoded(defined(*colour).clamp(Vec3::ZERO, Vec3::splat(f32::MAX)));

    // The engine's curve is not clamped: the encode of a value past one stays past one, as the present writes it.
    assert_close(shown[index], expected, &format!("{mode:?} {colour}"));
  }
}

/// PBR Neutral shows a colour under its shoulder as it is, less its toe's 0.04; AgX bleaches a saturated light towards
/// white as it brightens, where PBR Neutral keeps its hue.
#[test]
fn pbr_neutral_keeps_colour_and_agx_bleaches_bright_light() {
  let orange: Vec3 = Vec3::new(0.5, 0.2, 0.1);
  let cases: Vec<(Vec3, Vec4)> = vec![
    (orange, display(DisplayMode::PbrNeutral, 1.0, 1.0)),
    (orange, display(DisplayMode::Agx, 1.0, 1.0)),
    (orange * 16.0, display(DisplayMode::Agx, 1.0, 1.0)),
  ];
  let Some(shown) = show(&cases) else {
    return;
  };
  let chroma = |encoded: Vec3| (encoded.max_element() - encoded.min_element()) / encoded.max_element();

  assert_close(
    shown[0],
    encoded(orange - Vec3::splat(0.04)),
    "PBR Neutral under its shoulder",
  );
  assert!(
    chroma(shown[2]) < chroma(shown[1]) * 0.6,
    "AgX bleaches: {} at sixteen times, {} at one",
    chroma(shown[2]),
    chroma(shown[1])
  );
}

/// Grading of one leaves the light as it is; contrast pivots on middle grey; saturation of nought leaves the luminance
/// alone, grey.
#[test]
fn the_grading_pivots_on_middle_grey_and_keeps_luminance() {
  let colour: Vec3 = Vec3::new(0.41, 0.21, 0.03);
  let grey: Vec3 = Vec3::splat(0.18);
  let cases: Vec<(Vec3, Vec4)> = vec![
    (colour, display(DisplayMode::PbrNeutral, 1.0, 1.0)),
    (grey, display(DisplayMode::PbrNeutral, 1.6, 1.0)),
    (colour, display(DisplayMode::PbrNeutral, 1.3, 0.5)),
    (colour, display(DisplayMode::PbrNeutral, 1.0, 0.0)),
  ];
  let Some(shown) = show(&cases) else {
    return;
  };

  assert_close(shown[0], encoded(pbr_neutral(colour)), "ungraded");
  assert_close(shown[1], encoded(pbr_neutral(grey)), "middle grey under contrast");
  assert_close(shown[2], encoded(pbr_neutral(graded(colour, 1.3, 0.5))), "graded");
  assert!(
    (shown[3].x - shown[3].z).abs() < 1e-3,
    "saturation nought is grey: {}",
    shown[3]
  );
}
