use std::time::{Duration, Instant};

use glam::Vec3;

use crate::host::render_flare::RenderFlare;
use crate::host::render_lens_flare::RenderLensFlare;
use crate::pass::flare_uniform::{FLARE_SLOTS, FlareUniform};
use crate::scene::level::lens_flare_fade::LensFlareFade;

fn new_flare(rise_time: f32, down_time: f32, flares: usize) -> RenderLensFlare {
  RenderLensFlare {
    sprite: None,
    flares: (0..flares)
      .map(|index| RenderFlare {
        texture: format!("fx\\flare_{index}"),
        radius: 0.1,
        opacity: 0.5,
        position: index as f32,
      })
      .collect(),
    gradient: Some(RenderFlare {
      texture: "fx\\gradient".to_owned(),
      radius: 1.2,
      opacity: 0.4,
      position: 1.0,
    }),
    rise_time,
    down_time,
  }
}

fn find(name: &str) -> Option<RenderLensFlare> {
  match name {
    "day" => Some(new_flare(10.0, 10.0, 2)),
    "moon" => Some(new_flare(20.0, 20.0, 0)),
    _ => None,
  }
}

// `blend_rise_time` and `blend_down_time` are game seconds: a factor of ten runs them ten times faster.
#[test]
fn fades_a_lens_flare_in_over_its_rise_time_at_the_clock_rate() {
  let start: Instant = Instant::now();
  let mut fade: LensFlareFade = LensFlareFade::new();

  fade.advance(Some("day"), find, start, 10.0);
  fade.advance(Some("day"), find, start + Duration::from_millis(500), 10.0);

  let (name, blend) = fade.get_shown();

  assert_eq!(name, Some("day"));
  assert!((blend - 0.5).abs() < 1e-3, "{blend}");

  fade.advance(Some("day"), find, start + Duration::from_secs(2), 10.0);

  assert_eq!(fade.get_shown().1, 1.0);
}

// The engine hides the lens flare shown before the next one rises, each over its own time.
#[test]
fn hides_the_lens_flare_shown_before_the_next_one_rises() {
  let start: Instant = Instant::now();
  let mut fade: LensFlareFade = LensFlareFade::new();

  fade.advance(Some("day"), find, start, 0.0);
  fade.advance(Some("day"), find, start, 0.0);
  fade.advance(Some("day"), find, start, 0.0);
  fade.advance(Some("moon"), find, start + Duration::from_secs(1), 10.0);
  fade.advance(Some("moon"), find, start + Duration::from_millis(1500), 10.0);

  assert_eq!(fade.get_shown().0, Some("day"));
  assert!((fade.get_shown().1 - 0.5).abs() < 1e-3);

  fade.advance(Some("moon"), find, start + Duration::from_secs(3), 10.0);

  assert_eq!(fade.get_shown(), (Some("moon"), 0.0));
}

// A paused clock never moves, so a fade waiting on it would never end; it completes at once instead.
#[test]
fn completes_every_fade_at_once_while_the_clock_is_paused() {
  let start: Instant = Instant::now();
  let mut fade: LensFlareFade = LensFlareFade::new();

  for _ in 0..3 {
    fade.advance(Some("day"), find, start, 0.0);
  }

  assert_eq!(fade.get_shown(), (Some("day"), 1.0));
}

#[test]
fn writes_the_flares_and_gradient_a_lens_flare_draws() {
  let flare: RenderLensFlare = new_flare(1.0, 1.0, FLARE_SLOTS + 4);
  let uniform: FlareUniform = FlareUniform::new(
    &flare,
    (Vec3::new(0.0, 0.0, -1.0), Vec3::Y),
    Vec3::new(0.9, 0.8, 0.7),
    0.25,
    0.016,
  );

  assert_eq!(uniform.color.w, FLARE_SLOTS as f32);
  assert_eq!(uniform.flares[3].x, 3.0);
  assert_eq!(uniform.sun.w, 0.25);
  assert_eq!(uniform.to_sun.w, 0.016);
  assert_eq!(
    (uniform.gradient.x, uniform.gradient.y, uniform.gradient.z),
    (1.2, 0.4, 1.0)
  );
}
