//! RCAS's sharpening (`common/rcas`) on the GPU, over grey texels of the reversible tonemap a linear-light frame is
//! sharpened in: what it answers must undo to finite light, as the upscale pass undoes it.

use glam::Vec4;

use crate::contract::feature::image::resolve::render_upscaling_settings::RenderUpscalingSettings;
use crate::tests::shader::wgsl_probe::run_wgsl_probe;

/// The texel `e` among the four neighbours `ring` at `sharpness`.
struct RcasCase {
  ring: [f32; 4],
  e: f32,
  sharpness: f32,
}

/// Each case sharpened as a display-encoded frame is (`x`) and as a linear-light one is (`y`).
fn sharpen(cases: &[RcasCase]) -> Option<Vec<Vec4>> {
  let probes: Vec<(Vec4, Vec4)> = cases
    .iter()
    .map(|case| {
      (
        Vec4::new(case.ring[0], case.ring[1], case.e, case.ring[2]),
        Vec4::new(case.ring[3], case.sharpness, 0.0, 0.0),
      )
    })
    .collect();

  run_wgsl_probe(
    "common/rcas",
    "vec4<f32>(
      rcas_sharpen(vec3<f32>(probe.first.x), vec3<f32>(probe.first.y), vec3<f32>(probe.first.z),
        vec3<f32>(probe.first.w), vec3<f32>(probe.second.x), probe.second.y).x,
      rcas_sharpen_held(vec3<f32>(probe.first.x), vec3<f32>(probe.first.y), vec3<f32>(probe.first.z),
        vec3<f32>(probe.first.w), vec3<f32>(probe.second.x), probe.second.y).x,
      0.0, 0.0)",
    &probes,
  )
}

/// A texel of the reversible tonemap undone to linear light, as the upscale pass's `stored` undoes it.
fn undone(shown: f32) -> f32 {
  shown / (1.0 - shown).max(1e-4)
}

// A lamp far brighter than its dark surrounding, and a dark texel among bright ones, at the default
// sharpening and at the most: the sharpened texel stays within the tonemap's range, where the upscale pass undoes it
// to finite light, and a linear frame's is held near enough its brightest tap to undo to light of its order.
#[test]
fn rcas_keeps_a_sharpened_texel_within_the_range_it_undoes_from() {
  let default: f32 = RenderUpscalingSettings::default().get_sharpness();
  let mut cases: Vec<RcasCase> = Vec::new();

  for sharpness in [default, 1.0] {
    cases.push(RcasCase {
      ring: [0.05; 4],
      e: 0.9,
      sharpness,
    });
    cases.push(RcasCase {
      ring: [0.5; 4],
      e: 0.02,
      sharpness,
    });
    cases.push(RcasCase {
      ring: [0.05, 0.3, 0.6, 0.1],
      e: 0.95,
      sharpness,
    });
  }

  let Some(answered) = sharpen(&cases) else {
    return;
  };

  for (answer, case) in answered.iter().zip(&cases) {
    let brightest: f32 = case.ring.iter().copied().fold(case.e, f32::max);

    assert!(
      (0.0..=1.0).contains(&answer.x),
      "{} among {:?} at {}: shown {}",
      case.e,
      case.ring,
      case.sharpness,
      answer.x
    );
    assert!(
      answer.y >= 0.0 && undone(answer.y) <= (undone(brightest) * 2.0 + 1.0) * 1.0001,
      "{} among {:?} at {}: {} undone to {} against the brightest tap's {}",
      case.e,
      case.ring,
      case.sharpness,
      answer.y,
      undone(answer.y),
      undone(brightest)
    );
  }
}

// A flat neighbourhood is left as it is at any sharpening, and a bright texel among dark ones is still sharpened.
#[test]
fn rcas_leaves_a_flat_area_and_crisps_an_edge() {
  let default: f32 = RenderUpscalingSettings::default().get_sharpness();
  let cases: [RcasCase; 3] = [
    RcasCase {
      ring: [0.4; 4],
      e: 0.4,
      sharpness: default,
    },
    RcasCase {
      ring: [0.4; 4],
      e: 0.4,
      sharpness: 1.0,
    },
    RcasCase {
      ring: [0.1; 4],
      e: 0.3,
      sharpness: default,
    },
  ];
  let Some(answered) = sharpen(&cases) else {
    return;
  };

  for answer in &answered[..2] {
    assert!(
      (answer.x - 0.4).abs() < 1e-6 && (answer.y - 0.4).abs() < 1e-6,
      "{answer}"
    );
  }

  assert!(answered[2].x > 0.3 && answered[2].y == answered[2].x, "{}", answered[2]);
}
