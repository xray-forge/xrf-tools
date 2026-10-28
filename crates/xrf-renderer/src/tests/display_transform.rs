//! The physical light's display transforms and grading run on the GPU as the present runs them, held to their published
//! definitions (`pass/display_curves.rs`): Khronos PBR Neutral's reference, three.js's AgX, and the engine's curve; and
//! the transforms' anchors solved from them.

use glam::{Vec3, Vec4};

use crate::lighting::luminance::luminance;
use crate::lighting::srgb::srgb_encoded;
use crate::pass::display_curves::{agx, engine_display, get_least_shown, pbr_neutral};
use crate::pass::display_mode::DisplayMode;
use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_context::create_test_context;

/// A probe of `common/display_transform`: each case's colour and display in, `ANSWER` over them out.
const PROBE: &str = r"
struct DisplayCase {
  colour: vec4<f32>,
  display: vec4<f32>,
};

@group(0) @binding(0) var<storage, read_write> cases: array<DisplayCase>;

@compute @workgroup_size(1)
fn cs_display(@builtin(global_invocation_id) id: vec3<u32>) {
  let probe: DisplayCase = cases[id.x];

  cases[id.x].colour = vec4<f32>(ANSWER, 1.0);
}
";

/// The grading before a physical transform: luminance to `contrast` in stops about middle grey, colour mixed from it by
/// `saturation`.
fn graded(light: Vec3, contrast: f32, saturation: f32) -> Vec3 {
  let luminance_was: f32 = luminance(light).max(1e-6);
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
  run_probe("shown_for_display(probe.colour.rgb, probe.display)", cases)
}

/// What the GPU answers each case with under `answer`, an expression over the case's `probe.colour.rgb` and
/// `probe.display`.
fn run_probe(answer: &str, cases: &[(Vec3, Vec4)]) -> Option<Vec<Vec3>> {
  let context = create_test_context("to run the display transforms")?;
  let (device, queue) = (&context.device, &context.queue);
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let library: ShaderLibrary = ShaderLibrary::default();
  let source: String = library.compose("common/display_transform").unwrap() + &PROBE.replace("ANSWER", answer);
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

/// AgX's and PBR Neutral's anchors show the land's colour at the frame's key, graded, as bright as the engine's curve
/// shows it, from a night's key to a bright day's, under a soft, a neutral and a hard contrast.
#[test]
fn the_anchors_show_the_key_as_the_engines_curve_does() {
  let daylit: f32 = ExposureHistogramUniform::get_daylit_key(ExposureHistogramUniform::get_daylight_scale(0.0));
  let shown = |colour: Vec3| srgb_encoded(luminance(colour));

  for contrast in [0.7, 1.0, 1.5] {
    for key in [0.0005, 0.005, daylit, 0.3] {
      let hue: Vec3 = DisplayMode::ANCHOR_HUE;
      let land: Vec3 = graded(hue * (key / luminance(hue)), contrast, 1.0);
      let engine: f32 = shown(engine_display(land));

      for (mode, defined) in [
        (DisplayMode::Agx, agx as Reference),
        (DisplayMode::PbrNeutral, pbr_neutral),
      ] {
        // Under the least the transform shows, the key is held there rather than matched.
        if luminance(land) < get_least_shown(defined, hue) {
          continue;
        }

        let anchored: f32 = shown(defined(land * mode.get_anchor_stops(key, contrast).exp2()));

        assert!(
          (anchored - engine).abs() < 0.002,
          "{mode:?} at {key} by {contrast}: {anchored} against {engine}"
        );
      }
    }
  }
}

/// A night's key under the least AgX shows is held there: its lift stays the lift at that key, so whatever the night
/// lights above the key stays out of the shoulder, while PBR Neutral, which shows the darkest land, keeps matching.
#[test]
fn a_key_under_the_transforms_black_is_held_at_it() {
  let least: f32 = get_least_shown(agx, DisplayMode::ANCHOR_HUE);

  assert!(least > 1e-4 && least < 1e-3, "AgX shows the land from {least}");
  assert_eq!(
    DisplayMode::Agx.get_anchor_stops(least / 30.0, 1.0),
    DisplayMode::Agx.get_anchor_stops(least / 2.0, 1.0)
  );
  assert!(DisplayMode::Agx.get_anchor_stops(least / 2.0, 1.0) < 3.0);
  assert!(get_least_shown(pbr_neutral, DisplayMode::ANCHOR_HUE) < 3e-5);
}

/// The anchors move smoothly with the key and stay inside their bounds from a key no display shows to one past white,
/// under every contrast the grading offers.
#[test]
fn the_anchors_stay_bounded_and_continuous_over_every_key() {
  for mode in [DisplayMode::Agx, DisplayMode::PbrNeutral] {
    for contrast in [0.5, 1.0, 2.0] {
      let stops: Vec<f32> = (0..=108)
        .map(|step| mode.get_anchor_stops((-23.25 + 0.25 * step as f32).exp2(), contrast))
        .collect();

      for pair in stops.windows(2) {
        assert!((pair[1] - pair[0]).abs() < 0.5, "{mode:?} by {contrast}: {pair:?}");
      }

      for stop in stops {
        assert!((-5.5..7.5).contains(&stop), "{mode:?} by {contrast}: {stop}");
      }
    }
  }
}

/// Each physical transform and the engine's curve show light as their definitions do, ungraded.
#[test]
fn the_transforms_show_light_as_their_definitions_do() {
  let modes: [(DisplayMode, Reference); 3] = [
    (DisplayMode::PbrNeutral, pbr_neutral),
    (DisplayMode::Agx, agx),
    (DisplayMode::EngineEncoded, engine_display),
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

/// PBR Neutral shows a colour under its shoulder as it is, a dark one too, keeping its chroma as the engine's curve
/// does; AgX bleaches a saturated light towards white as it brightens.
#[test]
fn pbr_neutral_keeps_colour_and_agx_bleaches_bright_light() {
  let orange: Vec3 = Vec3::new(0.5, 0.2, 0.1);
  let land: Vec3 = Vec3::new(0.012, 0.01, 0.005);
  let cases: Vec<(Vec3, Vec4)> = vec![
    (orange, display(DisplayMode::PbrNeutral, 1.0, 1.0)),
    (orange, display(DisplayMode::Agx, 1.0, 1.0)),
    (orange * 16.0, display(DisplayMode::Agx, 1.0, 1.0)),
    (land, display(DisplayMode::PbrNeutral, 1.0, 1.0)),
  ];
  let Some(shown) = show(&cases) else {
    return;
  };
  let chroma = |encoded: Vec3| (encoded.max_element() - encoded.min_element()) / encoded.max_element();
  assert_close(shown[0], encoded(orange), "PBR Neutral under its shoulder");
  assert!(
    chroma(shown[2]) < chroma(shown[1]) * 0.6,
    "AgX bleaches: {} at sixteen times, {} at one",
    chroma(shown[2]),
    chroma(shown[1])
  );
  assert!(
    chroma(shown[3]) < chroma(encoded(engine_display(land))) * 1.15,
    "PBR Neutral keeps the dark land's chroma: {} against the engine's {}",
    chroma(shown[3]),
    chroma(encoded(engine_display(land)))
  );
}

/// Grading of one leaves the light as it is; contrast pivots on middle grey; saturation of nought leaves the luminance
/// alone, grey; a transform's stops lift the graded light before it.
#[test]
fn the_grading_pivots_on_middle_grey_and_keeps_luminance() {
  let colour: Vec3 = Vec3::new(0.41, 0.21, 0.03);
  let grey: Vec3 = Vec3::splat(0.18);
  let cases: Vec<(Vec3, Vec4)> = vec![
    (colour, display(DisplayMode::PbrNeutral, 1.0, 1.0)),
    (grey, display(DisplayMode::PbrNeutral, 1.6, 1.0)),
    (colour, display(DisplayMode::PbrNeutral, 1.3, 0.5)),
    (colour, display(DisplayMode::PbrNeutral, 1.0, 0.0)),
    (colour, display(DisplayMode::PbrNeutral, 1.0, 1.0) + Vec4::W),
    (colour, display(DisplayMode::PbrNeutral, 1.3, 0.5) + Vec4::W),
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
  assert_close(shown[4], encoded(pbr_neutral(colour * 2.0)), "a stop up");
  assert_close(
    shown[5],
    encoded(pbr_neutral(graded(colour, 1.3, 0.5) * 2.0)),
    "a stop up after the grading",
  );
}

/// Under the transform's black the lift never shows the frame's bright end brighter than the engine's curve does: a
/// night's key under AgX's black is held there, and its moonlit bright end, lifted with it, would show stops over the
/// engine's; matched, the bright end shows as the engine's. With no bright end metered, or a key the transform shows,
/// the lift is the key's anchor.
#[test]
fn the_lift_never_shows_the_bright_end_brighter_than_the_engines_curve() {
  let (night, moonlit): (f32, f32) = (2e-5, 2e-3);
  let held: f32 = DisplayMode::Agx.get_anchor_stops(night, 1.0);
  let lift: f32 = DisplayMode::Agx.get_lift((night, Some(moonlit)), 1.0);
  let hue: Vec3 = DisplayMode::ANCHOR_HUE;
  let bright: Vec3 = hue * (moonlit / luminance(hue));
  let shown = |colour: Vec3| srgb_encoded(luminance(colour));

  assert!(lift < held, "{lift} against the held {held}");
  assert!((shown(agx(bright * lift.exp2())) - shown(engine_display(bright))).abs() < 0.002);
  assert_eq!(DisplayMode::Agx.get_lift((night, None), 1.0), held);
  assert_eq!(DisplayMode::EngineEncoded.get_lift((night, Some(moonlit)), 1.0), 0.0);

  // Over the transform's range the key rules, whatever its bright end.
  let dusk: f32 = 5e-3;

  assert_eq!(
    DisplayMode::Agx.get_lift((dusk, Some(dusk * 256.0)), 1.0),
    DisplayMode::Agx.get_anchor_stops(dusk, 1.0)
  );
}

/// Display values across the range, greys and moderate colours as the game's pictures hold them.
fn shown_colours() -> Vec<Vec3> {
  vec![
    Vec3::splat(0.03),
    Vec3::splat(0.25),
    Vec3::splat(0.6),
    Vec3::splat(0.92),
    Vec3::new(0.6, 0.5, 0.4),
    Vec3::new(0.25, 0.32, 0.55),
    Vec3::new(0.9, 0.84, 0.78),
    Vec3::new(0.05, 0.06, 0.09),
  ]
}

/// `scene_for_display` undoes `shown_for_display`: the light it gives back is shown as the value it was given, under
/// the engine's curve, AgX and PBR Neutral, whatever the grading's contrast and saturation and the transform's lift.
#[test]
fn the_display_inverse_shows_what_it_was_given() {
  let mut cases: Vec<(Vec3, Vec4)> = Vec::new();

  for mode in [DisplayMode::EngineEncoded, DisplayMode::Agx, DisplayMode::PbrNeutral] {
    for (contrast, saturation, lift) in [(1.0, 1.0, 0.0), (0.8, 1.2, 2.0), (1.3, 0.8, -1.0)] {
      for colour in shown_colours() {
        cases.push((colour, Vec4::new(mode.get_index() as f32, contrast, saturation, lift)));
      }
    }
  }

  let Some(shown) = run_probe(
    "shown_for_display(scene_for_display(probe.colour.rgb, probe.display), probe.display)",
    &cases,
  ) else {
    return;
  };

  for ((colour, display), shown) in cases.iter().zip(shown) {
    assert!(
      (shown - *colour).abs().max_element() < 2e-3,
      "{display}: {shown} against {colour}"
    );
  }
}

/// A picture of the game's enters the scene so that the frame's display shows it as the engine's curve shows its light,
/// under AgX and PBR Neutral whatever lifts and grades the frame, as a display-referred plate comes in through the
/// inverse of its view; under the engine's curve it enters as it stands.
#[test]
fn a_picture_shows_under_every_display_as_the_engines_curve_shows_it() {
  let mut cases: Vec<(Vec3, Vec4)> = Vec::new();

  for mode in [DisplayMode::EngineEncoded, DisplayMode::Agx, DisplayMode::PbrNeutral] {
    for (contrast, saturation, lift) in [(1.0, 1.0, 0.0), (1.2, 1.1, 2.5)] {
      for colour in shown_colours() {
        cases.push((colour, Vec4::new(mode.get_index() as f32, contrast, saturation, lift)));
      }
    }
  }

  let shown_entered: &str = "shown_for_display(untonemapped(srgb_decoded(picture_for_display(probe.colour.rgb, \
                             probe.display)), 1.0), probe.display)";
  let engine_showing: &str = "tonemap(srgb_encoded(untonemapped(srgb_decoded(probe.colour.rgb), 1.0)), 1.0)";
  let unchanged: &str = "picture_for_display(probe.colour.rgb, probe.display)";
  let (Some(shown), Some(engine), Some(entered)) = (
    run_probe(shown_entered, &cases),
    run_probe(engine_showing, &cases),
    run_probe(unchanged, &cases),
  ) else {
    return;
  };

  for (index, (colour, display)) in cases.iter().enumerate() {
    if display.x as u32 == DisplayMode::EngineEncoded.get_index() {
      assert!(
        (entered[index] - *colour).abs().max_element() < 1e-6,
        "{display}: entered changed"
      );
    } else {
      assert!(
        (shown[index] - engine[index]).abs().max_element() < 2e-3,
        "{display}: {} against the engine's {}",
        shown[index],
        engine[index]
      );
    }
  }
}

/// `picture_from_display` takes a picture back out of the display it entered by, so a reader of the seen sky as light
/// gets the light the engine's curve shows, as the cube's sky is, under every display, grading and lift.
#[test]
fn a_picture_comes_back_out_of_the_display_it_entered_by() {
  let mut cases: Vec<(Vec3, Vec4)> = Vec::new();

  for mode in [DisplayMode::EngineEncoded, DisplayMode::Agx, DisplayMode::PbrNeutral] {
    for (contrast, saturation, lift) in [(1.0, 1.0, 0.0), (1.2, 1.1, 2.5), (0.8, 0.9, -1.5)] {
      for colour in shown_colours() {
        cases.push((colour, Vec4::new(mode.get_index() as f32, contrast, saturation, lift)));
      }
    }
  }

  let Some(back) = run_probe(
    "picture_from_display(picture_for_display(probe.colour.rgb, probe.display), probe.display)",
    &cases,
  ) else {
    return;
  };

  for ((colour, display), back) in cases.iter().zip(back) {
    assert!(
      (back - *colour).abs().max_element() < 2e-3,
      "{display}: {back} against {colour}"
    );
  }
}
