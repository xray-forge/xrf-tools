use std::f32::consts::FRAC_PI_2;

use glam::{Vec3, Vec4};

use crate::lighting::render_rainfall::RenderRainfall;
use crate::lighting::render_wind::RenderWind;

/// `max_desired_items`: streaks at the heaviest rain; half as many at the lightest.
pub const RAIN_STREAKS: u32 = 2500;

/// `drop_max_angle`: how far the strongest wind leans the streaks from the vertical, ten degrees.
const MAX_LEAN: f32 = 10.0 * std::f32::consts::PI / 180.0;

/// `drop_max_wind_vel`: the wind that leans them that far.
const MAX_LEAN_WIND: f32 = 20.0;

/// What `shaders/frame/rain.wgsl` reads as its `Rain`, as `dxRainRender::Render` sets the rain up.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RainUniform {
  /// `rain_color`, then the streaks' cover, `factor / 2 + .5`.
  pub color: Vec4,
  /// The way the rain falls, in renderer space.
  pub axis: Vec4,
  /// The cover's centre in `x` and `z`, its half width, and the height it is seen from.
  pub window: Vec4,
  /// Streaks drawn.
  pub count: u32,
  /// Seconds the rain has fallen.
  pub time: f32,
  /// Indices the splash's model draws.
  pub splash_indices: u32,
  pub pad: u32,
}

impl RainUniform {
  /// `strength` is `wind_strength_factor`, a tenth of which leans the streaks with the wind (`CEffect_Rain::Born`).
  pub fn new(
    rain: &RenderRainfall,
    (wind, strength): (RenderWind, f32),
    window: Vec4,
    time: f32,
    splash_indices: u32,
  ) -> Self {
    let density: f32 = rain.density.clamp(0.0, 1.0);
    // todo: Lean the rain as Monolith's `CEffect_Rain::Prepare` does on its engine, by the wind's velocity alone.
    let lean: f32 = (wind.velocity * strength / 10.0 / MAX_LEAN_WIND).clamp(0.0, 1.0);
    let pitch: f32 = MAX_LEAN * lean - FRAC_PI_2;
    let heading: f32 = wind.direction;
    // `axis.setHP(wind_direction, pitch)`, engine `z` negated into renderer space.
    let axis: Vec3 = Vec3::new(
      -pitch.cos() * heading.sin(),
      pitch.sin(),
      -(pitch.cos() * heading.cos()),
    );

    Self {
      color: rain.color.clamp(Vec3::ZERO, Vec3::ONE).extend(density / 2.0 + 0.5),
      axis: axis.extend(0.0),
      window,
      count: Self::get_count(density),
      time,
      splash_indices,
      pad: 0,
    }
  }

  /// Streaks drawn at a density, `0.5 * (1 + factor) * max_desired_items`.
  pub fn get_count(density: f32) -> u32 {
    (0.5 * (1.0 + density.clamp(0.0, 1.0)) * RAIN_STREAKS as f32) as u32
  }
}
