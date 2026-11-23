use std::cmp::Ordering;

use xrf_math::glam::{Vec3, Vec4};
use xrf_math::{Vector3d, glam};
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};
use xrf_visual::{LightDescription, LightKind};

use crate::camera::camera_view::CameraView;
use crate::contract::feature::lighting::render_shadow_settings::RenderShadowSettings;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::lighting::lamp::light_basis::LightBasis;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::lights::atlas::level_light_shadows::LevelLightShadows;
use crate::scene::level::lights::atlas::light_face_rank::LightFaceRank;
use crate::scene::level::lights::atlas::light_shadow_ask::LightShadowAsk;
use crate::scene::level::shadow::shadow_frame::ShadowFrame;
use crate::scene::level::shadow::shadow_sway::ShadowSway;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::tests::support::test_context::create_test_context;
use crate::tests::support::test_random::TestRandom;
use crate::tests::support::test_time::at;

// Near the eye, wide and long enough to want the largest square.
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
    is_detector_hidden: false,
    volumetric: None,
  }
}

// Fifteen spots take fifteen of the atlas's sixteen largest squares, leaving the point light a quarter size until
// the spots leave view.
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

// More lights than the atlas holds come into view and leave it, kept ready out of it, drawn, shrunk, grown and
// evicted, with frames abandoned and the lights forgotten between: `begin` asserts every frame that the squares in use
// are the lights' faces, and once the lights are forgotten the atlas is whole again.
#[test]
fn gives_back_every_square_through_frames_of_lights_coming_and_going() {
  let Some(context) = create_test_context("to hold the lights' shadow atlas") else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let mut shadows: LevelLightShadows = LevelLightShadows::new(device, &ViewBinding::create_layout(device));
  let statics: StaticScene = StaticScene::new(device, queue);
  let settings: RenderShadowSettings = RenderShadowSettings::default();
  let camera: CameraView = CameraView::new(
    Vec3::ZERO,
    glam::camera::rh::view::look_at_mat4(Vec3::ZERO, Vec3::Z, Vec3::Y),
    60.0,
    1.0,
    0.1,
    1000.0,
  );
  let mut handles: ProxyAllocator<LightDescription> = ProxyAllocator::new();
  // Nearest first, as a view asks for them; the points six faces each, so a few fill the atlas.
  let lights: Vec<(ProxyHandle<LightDescription>, LightDescription)> = (0..24)
    .map(|index| {
      let kind: LightKind = if index % 2 == 0 {
        LightKind::Spot
      } else {
        LightKind::Point
      };

      (handles.allocate(), light(kind, index as f32 * 1.5))
    })
    .collect();
  let mut random: TestRandom = TestRandom::new(0x9e37_79b9_7f4a_7c15);

  for frame in 1..=120 {
    let in_view: Vec<bool> = lights.iter().map(|_| random.next(3) == 0).collect();
    let sway: ShadowSway<'_> = ShadowSway {
      amplitude: 0.0,
      reach: 0.0,
      time: at(f64::from(frame) / 30.0),
      places: &[],
    };
    let mut scheduler: PeriodicScheduler = PeriodicScheduler::new();
    let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));

    shadows.begin();
    shadows.cast();
    lights
      .iter()
      .zip(&in_view)
      .filter(|(_, is_in_view)| **is_in_view)
      .for_each(|((handle, _), _)| shadows.see(*handle));

    // Every light in view first, then a few kept ready out of it.
    for is_in_view in [true, false] {
      for ((handle, light), _) in lights.iter().zip(&in_view).filter(|(_, seen)| **seen == is_in_view) {
        if !is_in_view && random.next(4) > 0 {
          continue;
        }

        let basis: LightBasis = LightBasis::of(light);

        shadows.ask(&LightShadowAsk {
          handle: *handle,
          light,
          basis: &basis,
          eye: Vec3::ZERO,
          forward: Vec3::Z,
          is_in_view,
          is_driving: true,
          statics: &statics,
          sway: &sway,
          casters: &[],
        });
      }
    }

    scheduler.begin(1000.0, &[], &[]);
    shadows.finish(&mut scheduler);
    scheduler.grant();
    shadows.prepare(
      device,
      &mut encoder,
      &ShadowFrame {
        scene: &statics,
        camera: &camera,
        settings: &settings,
        sun_direction: Vec3::NEG_Y,
        is_sunlit: true,
        switches: Vec4::ONE,
        sway,
        casters: &[],
      },
      &scheduler,
    );

    if random.next(4) == 0 {
      shadows.abandon_frame();
    }

    if frame % 50 == 0 {
      shadows.forget_lights();
    }
  }

  shadows.begin();
  shadows.forget_lights();

  assert_eq!(shadows.get_atlas_use().used, 0, "every square given back");
}

// A year in, `f32` seconds step by two and the two draws would tie.
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
