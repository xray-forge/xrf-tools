//! The camera's lens and film (`common/lens`), run on the GPU, and the physical camera Rust gives the camera effects
//! (`CameraEffectsUniform`).

use glam::{Mat4, Vec2, Vec4};

use crate::camera::camera_view::CameraView;
use crate::contract::render_camera_settings::RenderCameraSettings;
use crate::pass::camera_effects_uniform::CameraEffectsUniform;
use crate::tests::wgsl_probe::run_wgsl_probe;

fn run(answer: &str, cases: &[(Vec4, Vec4)]) -> Option<Vec<Vec4>> {
  run_wgsl_probe("common/lens", answer, cases)
}

#[test]
fn the_vignette_leaves_the_centre_and_dims_the_corners_by_cos_to_the_fourth() {
  let tangents: Vec4 = Vec4::new(0.75, 0.5, 0.0, 0.0);
  let cases: Vec<(Vec4, Vec4)> = [(0.5, 0.5), (1.0, 1.0), (0.0, 0.5)]
    .into_iter()
    .map(|(u, v)| (Vec4::new(u, v, 0.0, 0.0), tangents))
    .collect();
  let Some(answered) = run(
    "vec4<f32>(lens_vignette(probe.first.xy, probe.second.xy, 1.0), \
     lens_vignette(probe.first.xy, probe.second.xy, 0.5), lens_vignette(probe.first.xy, probe.second.xy, 0.0), 0.0)",
    &cases,
  ) else {
    return;
  };
  let full = |ray: Vec2| (1.0 / (1.0 + ray.length_squared())).powi(2);

  assert_eq!(answered[0].truncate(), glam::Vec3::ONE);
  assert!(
    (answered[1].x - full(Vec2::new(0.75, 0.5))).abs() < 1e-5,
    "{}",
    answered[1]
  );
  assert!(
    (answered[2].x - full(Vec2::new(0.75, 0.0))).abs() < 1e-5,
    "{}",
    answered[2]
  );

  // Half the strength stands halfway to the falloff; none leaves every pixel whole.
  for found in &answered {
    assert!((found.y - (1.0 + found.x) * 0.5).abs() < 1e-5, "{found}");
    assert_eq!(found.z, 1.0);
  }
}

#[test]
fn the_glass_magnifies_blue_most_and_red_least_and_nothing_without_strength() {
  let Some(answered) = run(
    "vec4<f32>(lens_magnifications(1.0), lens_magnifications(0.0).x)",
    &[(Vec4::ZERO, Vec4::ZERO)],
  ) else {
    return;
  };
  let [red, green, blue, unsplit] = answered[0].to_array();

  assert!(red < green && green < blue, "{}", answered[0]);
  assert_eq!(green, 1.0);
  assert_eq!(unsplit, 1.0);
  // Red to blue spans the whole spread at full strength.
  assert!((blue - red - 0.008).abs() < 1e-5, "{}", answered[0]);
}

#[test]
fn the_grain_stays_within_reach_averages_out_and_changes_every_frame() {
  let cases: Vec<(Vec4, Vec4)> = (0..512)
    .map(|index| {
      let pixel: Vec2 = Vec2::new((index % 32) as f32 * 1.37 + 3.0, (index / 32) as f32 * 1.91 + 7.0);

      (pixel.extend(0.0).extend(0.0), Vec4::ZERO)
    })
    .collect();
  let Some(answered) = run(
    "vec4<f32>(lens_grain(probe.first.xy, 1.6, 7u), lens_grain(probe.first.xy, 1.6, 8u), 0.0, 0.0)",
    &cases,
  ) else {
    return;
  };
  let mean: f32 = answered.iter().map(|found| found.x).sum::<f32>() / answered.len() as f32;
  let changed: usize = answered.iter().filter(|found| (found.x - found.y).abs() > 1e-3).count();

  assert!(
    answered
      .iter()
      .all(|found| found.x.abs() <= 1.0 && found.y.abs() <= 1.0)
  );
  assert!(mean.abs() < 0.1, "{mean}");
  assert!(
    changed * 10 >= answered.len() * 8,
    "{changed} of {} changed",
    answered.len()
  );
}

/// A view 60° high, 16 by 9, as `CameraView::new` makes it.
fn view() -> CameraView {
  CameraView::new(glam::Vec3::ZERO, Mat4::IDENTITY, 60.0, 16.0 / 9.0, 0.1, 1000.0)
}

#[test]
fn the_lens_takes_its_focal_length_from_the_view_and_its_sensor() {
  let settings: RenderCameraSettings = RenderCameraSettings {
    is_defocused: true,
    ..RenderCameraSettings::default()
  };
  let uniform: CameraEffectsUniform = CameraEffectsUniform::new(&settings, &view(), ((1920, 1080), (1920, 1080)));
  // A full frame's 24 mm across a half view of 30°: 12 mm over tan 30°.
  let focal: f32 = 0.012 / 30f32.to_radians().tan();

  assert!(uniform.is_focused());
  assert!((uniform.lens.y - focal).abs() < 1e-6, "{}", uniform.lens);
  assert!((uniform.lens.x - focal / 2.8).abs() < 1e-6, "{}", uniform.lens);
  assert_eq!(uniform.lens.z, 10.0);
  assert!((uniform.lens.w - 1080.0 / 0.024).abs() < 1e-2, "{}", uniform.lens);
}

#[test]
fn every_effect_is_off_by_default_and_parallel_rays_never_blur_out_of_focus() {
  let off: CameraEffectsUniform =
    CameraEffectsUniform::new(&RenderCameraSettings::default(), &view(), ((1920, 1080), (1920, 1080)));
  let orthographic: CameraView =
    CameraView::new_orthographic(glam::Vec3::ZERO, Mat4::IDENTITY, (50.0, 50.0), 0.1, 500.0);
  let mapped: CameraEffectsUniform = CameraEffectsUniform::new(
    &RenderCameraSettings {
      is_defocused: true,
      is_vignetted: true,
      ..RenderCameraSettings::default()
    },
    &orthographic,
    ((1024, 1024), (1024, 1024)),
  );

  assert!(!off.is_focused() && !off.is_motion_blurred() && !off.is_lensed());
  assert!(!mapped.is_focused());
  assert_eq!(mapped.optics.truncate().truncate(), Vec2::ZERO);
}

#[test]
fn a_motion_tile_spans_whole_blocks_and_reaches_the_longest_blur() {
  let settings: RenderCameraSettings = RenderCameraSettings {
    is_motion_blurred: true,
    ..RenderCameraSettings::default()
  };
  let uniform: CameraEffectsUniform = CameraEffectsUniform::new(&settings, &view(), ((1416, 965), (2830, 1930)));
  let tile: u32 = uniform.get_motion_tile_size();

  assert_eq!(tile % CameraEffectsUniform::MOTION_BLOCK, 0);
  // 4% of 1930 pixels is a radius of 77.2, which the tile of 80 holds whole.
  assert_eq!(tile, 80);
  assert!((uniform.motion.y - 77.2).abs() < 1e-3, "{}", uniform.motion);
  assert_eq!(uniform.drawn.truncate().truncate(), Vec2::new(1416.0, 965.0));
}
