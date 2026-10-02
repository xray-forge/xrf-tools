use std::f32::consts::{PI, SQRT_2};

use glam::{Vec3, Vec4};
use xrf_math::Vector3d;
use xrf_visual::{LightAnimatorDescription, LightAnimatorKey, LightDescription, LightKind};

use crate::lighting::light_animation::{to_animated_color, to_interpolated_color};
use crate::lighting::light_basis::{LightBasis, to_light_intensity, to_light_lod};
use crate::lighting::light_shadow_size::to_light_shadow_tile_size;

fn point() -> LightDescription {
  LightDescription {
    kind: LightKind::Point,
    position: Vector3d::new(1.0, 2.0, 3.0),
    direction: Vector3d::new(0.0, 0.0, -1.0),
    right: Vector3d::new(1.0, 0.0, 0.0),
    color: [1.0, 1.0, 1.0],
    range: 4.0,
    range_jitter: 0.0,
    cone: 0.0,
    near: 0.0,
    projector: None,
    animator: None,
    animator_scale: 0.0,
    is_shadowed: false,
    is_level: false,
  }
}

/// A spot down `-z` from the origin, `range` 10, with the cone given.
fn spot(cone: f32) -> LightDescription {
  LightDescription {
    kind: LightKind::Spot,
    position: Vector3d::new(0.0, 0.0, 0.0),
    range: 10.0,
    cone,
    ..point()
  }
}

fn near(a: f32, b: f32) -> bool {
  (a - b).abs() < 1e-4
}

#[test]
fn light_bounds_a_point_by_its_range_as_far_as_it_strays() {
  let light: LightDescription = LightDescription {
    range_jitter: 1.0,
    ..point()
  };
  let bound: Vec4 = LightBasis::of(&light).get_bound(&light);

  assert_eq!(bound, Vec4::new(1.0, 2.0, 3.0, 5.0));
}

#[test]
fn light_bounds_a_narrow_spot_through_its_apex_and_a_wide_one_by_its_rim() {
  let bound = |cone: f32| -> Vec4 {
    let light: LightDescription = spot(cone);

    LightBasis::of(&light).get_bound(&light)
  };
  let narrow: Vec4 = bound(PI / 4.0);
  let wide: Vec4 = bound(PI * 0.8);
  let below: Vec4 = bound(PI / 2.0 - 1e-4);
  let above: Vec4 = bound(PI / 2.0 + 1e-4);

  assert!(near(narrow.w, 10.0 / (2.0 * (PI / 8.0).cos())));
  assert!(near(narrow.z, -narrow.w));
  assert!(near(wide.w, 10.0 * (PI * 0.4).sin()));
  assert!((below.w - 10.0 / SQRT_2).abs() < 1e-3);
  assert!((above.w - below.w).abs() < 1e-2 && (above.z - below.z).abs() < 1e-2);
}

#[test]
fn light_takes_the_engines_spatial_sphere() {
  let narrow_light: LightDescription = spot(PI / 3.0);
  let wide_light: LightDescription = spot(PI * 2.0 / 3.0);
  let narrow: Vec4 = LightBasis::of(&narrow_light).get_spatial_sphere(&narrow_light);
  let wide: Vec4 = LightBasis::of(&wide_light).get_spatial_sphere(&wide_light);

  assert!(near(narrow.w, 10.0 / (2.0 * (PI / 6.0).cos().powi(2))));
  assert!(near(narrow.z, -narrow.w));
  assert!(near(wide.w, 10.0 * (PI / 3.0).tan()));
  assert!(near(wide.z, -10.0));
  assert_eq!(LightBasis::of(&point()).get_spatial_sphere(&point()).w, 4.0);
}

#[test]
fn light_basis_makes_the_given_right_square_to_the_direction() {
  let light: LightDescription = LightDescription {
    direction: Vector3d::new(0.0, -1.0, 0.0),
    right: Vector3d::new(1.0, 0.5, 0.0),
    ..spot(1.0)
  };
  let basis: LightBasis = LightBasis::of(&light);

  assert!(near(basis.right.x, 1.0) && near(basis.right.y, 0.0));
  assert!(near(basis.up.z, -1.0));
}

#[test]
fn light_basis_takes_the_worlds_up_or_forward_where_a_spot_gives_no_right() {
  let level: LightBasis = LightBasis::of(&LightDescription {
    right: Vector3d::new(0.0, 0.0, 0.0),
    ..spot(1.0)
  });
  let down: LightBasis = LightBasis::of(&LightDescription {
    direction: Vector3d::new(0.0, -1.0, 0.0),
    right: Vector3d::new(0.0, 0.0, 0.0),
    ..spot(1.0)
  });

  for basis in [level, down] {
    assert!(near(basis.right.length(), 1.0) && near(basis.up.length(), 1.0));
    assert!(near(basis.right.dot(basis.direction), 0.0) && near(basis.up.dot(basis.direction), 0.0));
  }

  assert!(near(level.up.y, 1.0));
  assert!(near(down.up.z.abs(), 1.0));
}

#[test]
fn light_fades_by_get_lod_between_the_thresholds() {
  let sphere: Vec4 = Vec4::new(0.0, 0.0, -10.0, 2.0);
  let area: f32 = 0.5 * 2.0 / 100.0;

  assert!(near(
    to_light_lod(sphere, Vec3::ZERO, 0.02, 0.001),
    ((area - 0.001) / (0.02 - 0.001)).sqrt()
  ));
  assert_eq!(to_light_lod(sphere, Vec3::ZERO, 0.002, 0.001), 1.0);
  assert_eq!(to_light_lod(sphere, Vec3::ZERO, 0.5, 0.1), 0.0);
}

#[test]
fn light_intensity_averages_the_colours_mean_and_luminance() {
  assert!(near(to_light_intensity(Vec3::ONE), 1.0));
  assert!(near(to_light_intensity(Vec3::Y), (1.0 / 3.0 + 0.7154) / 2.0));
}

#[test]
fn light_shadow_tiles_take_the_nearest_power_of_two_within_bounds() {
  assert_eq!(to_light_shadow_tile_size(700.0), 512);
  assert_eq!(to_light_shadow_tile_size(800.0), 1024);
  assert_eq!(to_light_shadow_tile_size(1536.0), 1024);
  assert_eq!(to_light_shadow_tile_size(5.0), 32);
}

fn animator() -> LightAnimatorDescription {
  LightAnimatorDescription {
    fps: 10.0,
    frame_count: 20,
    keys: vec![
      LightAnimatorKey {
        frame: 0,
        color: [0.0, 0.0, 0.0],
      },
      LightAnimatorKey {
        frame: 10,
        color: [200.0, 100.0, 50.0],
      },
      LightAnimatorKey {
        frame: 15,
        color: [100.0, 100.0, 100.0],
      },
    ],
  }
}

#[test]
fn light_animation_holds_a_keys_colour_and_blends_between_keys() {
  assert_eq!(to_interpolated_color(&animator(), 10.0), Vec3::new(200.0, 100.0, 50.0));
  assert_eq!(to_interpolated_color(&animator(), 5.0), Vec3::new(100.0, 50.0, 25.0));
  assert_eq!(to_interpolated_color(&animator(), 12.0), Vec3::new(160.0, 100.0, 70.0));
  assert_eq!(to_interpolated_color(&animator(), 19.0), Vec3::new(100.0, 100.0, 100.0));
}

#[test]
fn light_animation_steps_time_to_frames_and_loops() {
  assert_eq!(to_animated_color(&animator(), 0.55), Vec3::new(100.0, 50.0, 25.0));
  assert_eq!(to_animated_color(&animator(), 2.55), Vec3::new(100.0, 50.0, 25.0));
  assert_eq!(to_animated_color(&animator(), 1.0), Vec3::new(200.0, 100.0, 50.0));
}
