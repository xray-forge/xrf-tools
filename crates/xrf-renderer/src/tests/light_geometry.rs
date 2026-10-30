use std::f32::consts::{PI, SQRT_2};

use glam::{Vec3, Vec4};
use xrf_engine_target::XrayEngine;
use xrf_light_anim::{LightAnimItem, LightAnimKey};
use xrf_math::Vector3d;
use xrf_visual::{LightAnimatorDescription, LightAnimatorKey, LightDescription, LightKind, LightVolumetric};

use crate::contract::render_light_shafts::RenderLightShafts;
use crate::contract::render_lights_settings::RenderLightsSettings;
use crate::lighting::light_animation::{to_animated_color, to_interpolated_color};
use crate::lighting::light_basis::LightBasis;
use crate::lighting::light_shadow_size::to_light_shadow_tile_size;
use crate::scene::level::level_lights::LevelLights;

// Under the inverse square a light gives a point `r²/d²` of its colour, so an idle light whose range strays each frame
// would flash its whole pool; held by the ratio of its own range's square to the stray's, it gives every point inside
// what its own range gives, at every stray, and a light that does not stray is left alone.
#[test]
fn an_idle_lights_jitter_leaves_its_inverse_square_strength_alone() {
  let range: f32 = 4.0;
  let distance: f32 = 1.5;
  let given =
    |jittered: f32| LevelLights::to_jitter_hold(range, jittered) * jittered * jittered / (distance * distance);

  for jittered in [3.0_f32, 4.0, 5.0] {
    assert!(
      (given(jittered) - range * range / (distance * distance)).abs() < 1e-5,
      "{jittered}"
    );
  }

  assert_eq!(LevelLights::to_jitter_hold(range, range), 1.0);
  assert_eq!(LevelLights::to_jitter_hold(range, 0.0), 1.0);
}

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
    campfire: None,
    motion: None,
    zone_sphere: None,
    volumetric: None,
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

// A light's shafts add a metre of air what the engine's slices within its sphere add: 99 steps of the buffer over the
// depth its quality stretches, each a fortieth at the colour's scale. Under vanilla only a light that asks has them,
// the quality cancelling as the engine scales its colour up by it; under the extended engine every light has them, a
// point's at Monolith's part, gone where they have faded out far from the eye.
#[test]
fn light_shafts_scatter_what_the_engines_slices_add_a_metre() {
  let asking: LightDescription = LightDescription {
    volumetric: Some(LightVolumetric {
      is_asked: true,
      quality: 0.5,
      intensity: 0.4,
      distance: 1.0,
    }),
    ..point()
  };
  let silent: LightDescription = LightDescription {
    volumetric: Some(LightVolumetric {
      is_asked: false,
      ..asking.volumetric.clone().unwrap()
    }),
    ..point()
  };
  let density = |light: &LightDescription, eye: Vec3, engine: XrayEngine| {
    LevelLights::to_shafts(
      light,
      &LightBasis::of(light),
      eye,
      engine,
      &RenderLightsSettings::default(),
    )
    .map(|shafts| shafts.get_density())
  };

  assert!(near(
    density(&asking, Vec3::ZERO, XrayEngine::Vanilla).unwrap(),
    99.0 / 40.0 * 0.4 / 8.0
  ));
  assert_eq!(density(&silent, Vec3::ZERO, XrayEngine::Vanilla), None);
  assert_eq!(density(&point(), Vec3::ZERO, XrayEngine::Extended), None);
  assert!(near(
    density(&silent, Vec3::ZERO, XrayEngine::Extended).unwrap(),
    99.0 * 0.6 / 40.0 * 0.2 / 3.0 / 8.0
  ));
  assert_eq!(density(&silent, Vec3::new(0.0, 0.0, 200.0), XrayEngine::Extended), None);
}

// Asked for every shadowed light's shafts, a light that asks keeps its own values and one that does not, or has none,
// takes the engine's defaults; the strength scales them all, and with shafts off or at no strength none draw.
#[test]
fn light_shafts_follow_the_views_choice_and_strength() {
  let asking: LightDescription = LightDescription {
    volumetric: Some(LightVolumetric {
      is_asked: true,
      quality: 1.0,
      intensity: 0.4,
      distance: 1.0,
    }),
    ..point()
  };
  let density = |light: &LightDescription, settings: RenderLightsSettings| {
    LevelLights::to_shafts(
      light,
      &LightBasis::of(light),
      Vec3::ZERO,
      XrayEngine::Vanilla,
      &settings,
    )
    .map(|shafts| shafts.get_density())
  };
  let every: RenderLightsSettings = RenderLightsSettings {
    shafts: RenderLightShafts::Shadowed,
    ..RenderLightsSettings::default()
  };

  assert_eq!(density(&point(), RenderLightsSettings::default()), None);
  assert!(near(density(&point(), every).unwrap(), 99.0 / 40.0 / 8.0));
  assert!(near(density(&asking, every).unwrap(), 99.0 / 40.0 * 0.4 / 8.0));
  assert!(near(
    density(
      &asking,
      RenderLightsSettings {
        shaft_strength: 2.5,
        ..every
      }
    )
    .unwrap(),
    99.0 / 40.0 / 8.0
  ));
  assert_eq!(
    density(
      &asking,
      RenderLightsSettings {
        is_volumetric: false,
        ..every
      }
    ),
    None
  );
  assert_eq!(
    density(
      &asking,
      RenderLightsSettings {
        shaft_strength: 0.0,
        ..every
      }
    ),
    None
  );
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
    LightBasis::to_light_lod(sphere, Vec3::ZERO, 0.02, 0.001),
    ((area - 0.001) / (0.02 - 0.001)).sqrt()
  ));
  assert_eq!(LightBasis::to_light_lod(sphere, Vec3::ZERO, 0.002, 0.001), 1.0);
  assert_eq!(LightBasis::to_light_lod(sphere, Vec3::ZERO, 0.5, 0.1), 0.0);
}

#[test]
fn light_intensity_averages_the_colours_mean_and_luminance() {
  assert!(near(LightBasis::to_light_intensity(Vec3::ONE), 1.0));
  assert!(near(
    LightBasis::to_light_intensity(Vec3::Y),
    (1.0 / 3.0 + 0.7154) / 2.0
  ));
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

/// The renderer's colour is the engine's before `color_argb_f` floors it to a byte, so the two never differ by more
/// than the one step that floor takes off a channel.
fn assert_matches_the_engine(rendered: Vec3, engine: u32, at: &str) {
  let engine: Vec3 = Vec3::new(
    ((engine >> 16) & 0xFF) as f32,
    ((engine >> 8) & 0xFF) as f32,
    (engine & 0xFF) as f32,
  );

  assert!(
    (rendered - engine).abs().max_element() <= 1.0,
    "{at}: the renderer draws {rendered}, the engine {engine}"
  );
}

#[test]
fn light_animation_draws_what_the_engines_light_animation_computes() {
  // `svet_01`'s keys, the one every shipped library holds with a key before frame 0, beside an uneven flicker.
  let items: [LightAnimItem; 2] = [
    LightAnimItem {
      name: String::from("svet_01"),
      fps: 30.0,
      frame_count: 30,
      keys: [
        (-1, 0x00E9_CE72),
        (0, 0x00FF_C066),
        (15, 0x00FC_CA84),
        (29, 0x00FF_C066),
      ]
      .map(|(frame, color)| LightAnimKey { frame, color })
      .to_vec(),
    },
    LightAnimItem {
      name: String::from("flicker"),
      fps: 15.0,
      frame_count: 40,
      keys: [
        (0, 0xFF00_0000),
        (3, 0xFF7F_10FF),
        (4, 0xFF01_FE02),
        (17, 0xFF33_6699),
        (40, 0xFFFF_FFFF),
      ]
      .map(|(frame, color)| LightAnimKey { frame, color })
      .to_vec(),
    },
  ];

  for item in &items {
    let animator: LightAnimatorDescription = LightAnimatorDescription::of(item, false);

    for frame in 0..=item.frame_count as i32 {
      assert_matches_the_engine(
        to_interpolated_color(&animator, frame as f32),
        item
          .interpolate_rgb(frame)
          .expect("every frame from zero is keyed at or before"),
        &format!("{} frame {frame}", item.name),
      );
    }

    for step in 0..120 {
      let seconds: f32 = step as f32 * 0.037;
      let (_, color) = item.calculate_rgb(seconds).expect("the animation advances");

      assert_matches_the_engine(
        to_animated_color(&animator, seconds),
        color,
        &format!("{} at {seconds} s", item.name),
      );
    }
  }
}
