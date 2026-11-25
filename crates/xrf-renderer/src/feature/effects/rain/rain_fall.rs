use std::f32::consts::FRAC_PI_2;

use xrf_math::glam::{Vec2, Vec3};

use crate::feature::effects::rain::rain_frame::RainFrame;

/// `drop_max_angle` (Monolith's `Rain.cpp`): how far the strongest wind leans the drops, thirty five degrees.
const MAX_LEAN: f32 = 35.0 * std::f32::consts::PI / 180.0;

/// `drop_angle`: how far a drop strays at most in still air before its `1.5 - velocity`, fifteen degrees.
const STRAY: f32 = 15.0 * std::f32::consts::PI / 180.0;

/// `source_offset` and `max_distance`.
const SOURCE_OFFSET: f32 = 20.0;
const MAX_DISTANCE: f32 = SOURCE_OFFSET * 1.5;

/// How far ahead along the camera's way `Born` aims the drops, in metres.
const AIM: f32 = 5.0;

/// Seconds the fall's eased values take to follow most of the way: its square's aim, its pace and its lean.
const EASING: f32 = 1.0;

/// How the rain falls, as Monolith's `CEffect_Rain::Born` and `Prepare` send each drop off, whichever engine the level
/// is drawn as and whichever drops are drawn: leaned against its negated `wind_direction` by the wind's velocity and
/// the rain's own gust, straying less and falling faster the harder it blows, ahead of where the camera looks. What a
/// drop looks like is not the fall's.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RainFall {
  /// The way the rain falls, in renderer space.
  pub axis: Vec3,
  /// The widest a drop strays from it, in radians.
  pub stray: f32,
  /// Metres over the eye a drop starts, and the metres it falls at most.
  pub source_offset: f32,
  pub max_distance: f32,
  /// Metres a side of the square the drops fall over.
  pub side: f32,
  /// Where that square stands from the eye, in renderer `x` and `z`.
  pub aim: Vec2,
  /// How fast the drops fall against their own speeds, by the wind.
  pub pace: f32,
  /// The share of the most drops that fall at the rain's density, `0.01 * (1 + factor * 99)`
  /// (`dxRainRender::Render`).
  pub share: f32,
}

impl RainFall {
  /// `source_radius`, and `ssfx_rain_drops_setup`'s radius: `Born` places a drop within half of it either way, so the
  /// side of its square.
  pub(crate) const SIDE: f32 = 15.0;

  /// One `frame`'s fall.
  pub(crate) fn new(frame: &RainFrame<'_>) -> Self {
    let density: f32 = frame.rain.density.clamp(0.0, 1.0);
    let velocity: f32 = frame.wind.get_strength();
    let lean: f32 = (velocity + frame.gust.rain).clamp(0.0, 1.0);

    Self {
      axis: Self::get_axis(MAX_LEAN * lean, -frame.wind.direction),
      stray: STRAY * (1.5 - velocity),
      source_offset: SOURCE_OFFSET,
      max_distance: MAX_DISTANCE,
      side: Self::SIDE,
      aim: frame.aim,
      pace: (velocity * 1.5).clamp(0.5, 1.0),
      share: 0.01 * (1.0 + density * 99.0),
    }
  }

  /// Where the drops' square stands from the eye: `aim` eased `step` seconds towards `Born`'s aim along `look`
  /// (`mad(view, dir, 5)`, its height dropped); there at once without one.
  pub(crate) fn ease_aim(aim: Option<Vec2>, look: Vec3, step: f32) -> Vec2 {
    let target: Vec2 = Vec2::new(look.x, look.z) * AIM;

    aim.map_or(target, |aim| aim.lerp(target, Self::get_following(step)))
  }

  /// The share of the way to its target an eased value of the fall goes in `step` seconds.
  pub(crate) fn get_following(step: f32) -> f32 {
    1.0 - (-step / EASING).exp()
  }

  /// The drops falling of `most`, at the rain's density.
  pub(crate) fn get_count(&self, most: f32) -> u32 {
    (self.share * most) as u32
  }

  /// `axis.setHP(heading, pitch - PI / 2)` leaning `pitch` from straight down, engine `z` negated into renderer space.
  fn get_axis(pitch: f32, heading: f32) -> Vec3 {
    let pitch: f32 = pitch - FRAC_PI_2;

    Vec3::new(
      -pitch.cos() * heading.sin(),
      pitch.sin(),
      -(pitch.cos() * heading.cos()),
    )
  }
}
