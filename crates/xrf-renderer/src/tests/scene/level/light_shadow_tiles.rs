//! How the lights' shadow faces share the atlas under pressure, and in what order those wanting a draw are taken.

use glam::Vec3;
use xrf_math::Vector3d;
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};
use xrf_visual::{LightDescription, LightKind};

use std::cmp::Ordering;

use crate::lighting::lamp::light_basis::LightBasis;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::lights::atlas::level_light_shadows::LevelLightShadows;
use crate::scene::level::lights::atlas::light_face_rank::LightFaceRank;
use crate::scene::level::lights::atlas::light_shadow_ask::LightShadowAsk;
use crate::scene::level::shadow::shadow_sway::ShadowSway;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::tests::support::test_context::create_test_context;
use crate::tests::support::test_time::at;

/// A shadowed light near the eye, which looks down `+z`: wide enough, long and facing it to want the largest square.
fn light(kind: LightKind, x: f32) -> LightDescription {
  LightDescription {
    kind,
    position: Vector3d::new(x, 0.0, 2.0),
    direction: Vector3d::new(0.0, 0.0, -1.0),
    right: Vector3d::new(1.0, 0.0, 0.0),
    color: [1.0, 1.0, 1.0],
    range: 16.0,
    range_jitter: 0.0,
    cone: std::f32::consts::FRAC_PI_2,
    near: 0.0,
    projector: None,
    animator: None,
    animator_scale: 0.0,
    is_shadowed: true,
    is_level: false,
    campfire: None,
    motion: None,
    zone_sphere: None,
    volumetric: None,
  }
}

/// A light given smaller squares while every other light was in view takes squares of its own size again once room
/// can be made for them: fifteen spots take fifteen of the atlas's sixteen largest squares, a point light asked at the
/// largest gets its six faces at a quarter of it, and the next frame, the spots out of view, its ask grows it back.
#[test]
fn grows_a_light_given_smaller_squares_back_once_room_is_made() {
  let Some(context) = create_test_context("to hold the lights' shadow atlas") else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let mut shadows: LevelLightShadows = LevelLightShadows::new(device, &ViewBinding::create_layout(device));
  let statics: StaticScene = StaticScene::new(device, queue);
  let sway: ShadowSway<'_> = ShadowSway {
    amplitude: 0.0,
    reach: 0.0,
    time: at(1.0),
    places: &[],
  };
  let mut handles: ProxyAllocator<LightDescription> = ProxyAllocator::new();
  let spots: Vec<(ProxyHandle<LightDescription>, LightDescription)> = (0..15)
    .map(|index| (handles.allocate(), light(LightKind::Spot, index as f32 * 0.1)))
    .collect();
  let point: (ProxyHandle<LightDescription>, LightDescription) = (handles.allocate(), light(LightKind::Point, 0.0));
  let ask = |shadows: &mut LevelLightShadows, (handle, light): &(ProxyHandle<LightDescription>, LightDescription)| {
    let basis: LightBasis = LightBasis::of(light);

    shadows.ask(&LightShadowAsk {
      handle: *handle,
      light,
      basis: &basis,
      eye: Vec3::ZERO,
      forward: Vec3::Z,
      is_in_view: true,
      is_driving: true,
      statics: &statics,
      sway: &sway,
      casters: &[],
    });
  };
  let square = |side: u64| side * side;

  shadows.begin();
  spots.iter().for_each(|spot| ask(&mut shadows, spot));
  ask(&mut shadows, &point);

  assert_eq!(
    u64::from(shadows.get_atlas_use().used),
    15 * square(1024) + 6 * square(256),
    "the point light's faces at a quarter of its size, every other light in view"
  );

  shadows.begin();
  ask(&mut shadows, &point);

  // The smaller squares still held the one square left while the six were found, so six spots gave theirs.
  assert_eq!(
    u64::from(shadows.get_atlas_use().used),
    (9 + 6) * square(1024),
    "nine spots and its six faces at its own size, its smaller squares given back"
  );
}

/// Faces drawn a tenth of a second apart are taken the longest drawn first however long the level's clock has run: a
/// year in, seconds as `f32` step by two, and the two would tie.
#[test]
fn takes_the_face_drawn_longest_ago_first_a_year_into_the_level() {
  let year: f64 = 365.0 * 24.0 * 3600.0;
  let (earlier, later) = (
    LightFaceRank::DrawnAt(at(year + 0.1)),
    LightFaceRank::DrawnAt(at(year + 0.2)),
  );

  assert_eq!(earlier.compare(later), Ordering::Less);
  assert_eq!(later.compare(earlier), Ordering::Greater);
  assert_eq!(
    LightFaceRank::Distance(1.0).compare(LightFaceRank::Distance(2.0)),
    Ordering::Less,
    "a stale face by its light's distance, nearest first"
  );
}
