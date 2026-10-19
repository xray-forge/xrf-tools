use std::f32::consts::FRAC_PI_2;

use glam::{Vec2, Vec3, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::render_rain_drops_mode::RenderRainDropsMode;
use crate::lighting::render_rainfall::RenderRainfall;
use crate::lighting::render_wind::RenderWind;

/// `drop_max_angle`: how far the strongest wind leans the streaks from the vertical, ten degrees.
const MAX_LEAN: f32 = 10.0 * std::f32::consts::PI / 180.0;

/// `drop_max_wind_vel`: the wind that leans them that far.
const MAX_LEAN_WIND: f32 = 20.0;

/// `drop_angle`: how far a streak strays from the rain's way, three degrees.
const STRAY: f32 = 3.0 * std::f32::consts::PI / 180.0;

/// `source_offset`, `max_distance` and `source_radius` (as the side of a square of the disc's area).
const SOURCE_OFFSET: f32 = 40.0;
const MAX_DISTANCE: f32 = SOURCE_OFFSET * 1.25;
const SOURCE_TILE: f32 = 12.5 * 1.772_453_9;

/// `drop_length` and `drop_width`, which is half a streak's width.
const STREAK_LENGTH: f32 = 5.0;
const STREAK_WIDTH: f32 = 0.3;

/// `drop_speed_min` and `drop_speed_max`, in metres a second.
const SPEED_MIN: f32 = 40.0;
const SPEED_MAX: f32 = 80.0;

/// Monolith's `drop_max_angle` (`Rain.cpp`): how far its strongest wind leans the drops, thirty five degrees.
const ENHANCED_MAX_LEAN: f32 = 35.0 * std::f32::consts::PI / 180.0;

/// Monolith's `drop_angle`: how far a drop strays at most in still air before its `1.5 - velocity`, fifteen degrees.
const ENHANCED_STRAY: f32 = 15.0 * std::f32::consts::PI / 180.0;

/// Monolith's `source_offset` and `max_distance`.
const ENHANCED_SOURCE_OFFSET: f32 = 20.0;
const ENHANCED_MAX_DISTANCE: f32 = ENHANCED_SOURCE_OFFSET * 1.5;

/// How far ahead along the camera's way Monolith's `Born` aims the drops, in metres.
const ENHANCED_AIM: f32 = 5.0;

/// Seconds the enhanced drops' square takes to follow the camera's way most of the way round, about a drop's fall:
/// Monolith's drops stay where they were born and new ones are born ahead, where a square turned at once with the
/// camera moved most of its drops in one frame.
const ENHANCED_AIM_EASING: f32 = 1.0;

/// `ssfx_rain_drops_setup`: drops at the heaviest and its square's side. Each of its 2500 streaks carries two drops,
/// where each of ours carries one, falling on its own; and Monolith's drops are born again as they land, where a
/// stateless fall waits out its turn, about a quarter of it: so 6000 for its 5000 drops.
const ENHANCED_DROPS: f32 = 6000.0;
const ENHANCED_RADIUS: f32 = 15.0;

/// The share of landing drops that splash: `CEffect_Rain::Hit`'s half of Monolith's 2500, among the 6000.
const ENHANCED_SPLASHING: f32 = 0.21;

/// `ssfx_rain_1`'s half width and speed's scale, and a streak of one drop, as long as each of its two drops drew on
/// its 2 m streaks: a drop's fall over a frame's exposure, a few tenths of a metre at these speeds.
const ENHANCED_LENGTH: f32 = 0.55;
const ENHANCED_WIDTH: f32 = 0.1;
const ENHANCED_SPEED: f32 = 0.6;

/// `ssfx_rain_2`'s cover, brightness, refraction and reflection: G.A.M.M.A.'s refraction and reflection (its MCM
/// values), a cover well below its 0.65 as a motion-blurred drop's is, and the share of the light it scatters in
/// place of its brightness (0.205 of the hemisphere added), low enough that rain does not draw the eye.
const ENHANCED_LOOK: Vec4 = Vec4::new(0.35, 0.2, 1.2, 0.5);

/// The enhanced splash, a slight addition: how much the light it scatters covers, how far its map bends the ground
/// through the distortion target (in its units, which the window pass scales by `def_distort`), unused, and the share
/// of the landings that splash. SSS's `ssfx_rain_3` bent a blurred copy of the ground, which drew a soft grey blob.
const ENHANCED_SPLASH_LOOK: Vec4 = Vec4::new(0.08, 0.5, 0.0, ENHANCED_SPLASHING);

/// What `shaders/frame/rain.wgsl` reads as its `Rain`, as `dxRainRender::Render` sets the rain up.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Rain")]
pub struct RainUniform {
  /// `rain_color`, then `factor / 2 + .5`, which the streaks are drawn long by and the engine's covered by.
  pub color: Vec4,
  /// The way the rain falls, in renderer space, then the widest a drop strays from it, in radians.
  pub axis: Vec4,
  /// The cover's centre in `x` and `z`, its half width, and the height it is seen from; then its texels across and
  /// the metres it reaches down, the rest unused.
  pub window: Vec4,
  pub cover_shape: Vec4,
  /// Metres over the eye a drop starts, metres it falls at most and the side of the square it falls over; the rest
  /// unused.
  pub fall: Vec4,
  /// A drop's length at the heaviest rain and half its width, then its slowest and fastest speed in metres a second.
  pub streak: Vec4,
  /// The enhanced drops' cover, the share of the light they scatter, refraction and reflection; then the splashes'
  /// cover of the light they scatter, bend of the ground, unused, and share of the landings that splash.
  pub look: Vec4,
  pub splash_look: Vec4,
  /// Where the enhanced drops' square stands from the eye, in `x` and `z`, then the texels across and down of the
  /// image the enhanced streaks draw over.
  pub aim: Vec4,
  /// Drops drawn.
  pub count: u32,
  /// Seconds the rain has fallen.
  pub time: f32,
  /// Indices the splash's model draws.
  pub splash_indices: u32,
  /// One where the drops are the enhanced ones, straying as Monolith's `random_dir` scatters them.
  pub is_enhanced: u32,
}

impl RainUniform {
  /// `max_desired_items`: streaks at the heaviest rain; half as many at the lightest.
  pub const STREAKS: u32 = 2500;

  /// `strength` is `wind_strength_factor`: a tenth of it leans the engine's streaks with the wind
  /// (`CEffect_Rain::Born`), and its swing about a half stands in for the gust Monolith's `CEffect_Rain::Prepare` leans
  /// its drops by.
  pub fn new(
    (rain, drops): (&RenderRainfall, RenderRainDropsMode),
    (wind, strength): (RenderWind, f32),
    (window, cover_shape): (Vec4, Vec4),
    (time, aim): (f32, Vec2),
    splash_indices: u32,
  ) -> Self {
    let density: f32 = rain.density.clamp(0.0, 1.0);
    let base: Self = Self {
      color: rain.color.clamp(Vec3::ZERO, Vec3::ONE).extend(density / 2.0 + 0.5),
      window,
      cover_shape,
      time,
      splash_indices,
      ..Default::default()
    };

    match drops {
      RenderRainDropsMode::Engine => {
        // todo: Lean the rain as Monolith's `CEffect_Rain::Prepare` does on its engine, by the wind's velocity alone.
        let lean: f32 = (wind.velocity * strength / 10.0 / MAX_LEAN_WIND).clamp(0.0, 1.0);

        Self {
          axis: Self::get_axis(MAX_LEAN * lean, wind.direction).extend(STRAY),
          fall: Vec4::new(SOURCE_OFFSET, MAX_DISTANCE, SOURCE_TILE, 0.0),
          streak: Vec4::new(STREAK_LENGTH, STREAK_WIDTH, SPEED_MIN, SPEED_MAX),
          count: Self::get_count(density),
          ..base
        }
      }
      RenderRainDropsMode::Enhanced => {
        // Monolith reads `wind_velocity` over a thousand; its noise swings the lean by up to twice its own two thirds,
        // where `wind_strength_factor` swings by one about a half.
        let velocity: f32 = (wind.velocity * 0.001).clamp(0.0, 1.0);
        let lean: f32 = (velocity + (strength - 0.5) * 2.0).clamp(0.0, 1.0);
        let speed: f32 = ENHANCED_SPEED * (velocity * 1.5).clamp(0.5, 1.0);

        Self {
          axis: Self::get_axis(ENHANCED_MAX_LEAN * lean, wind.direction).extend(ENHANCED_STRAY * (1.5 - velocity)),
          fall: Vec4::new(ENHANCED_SOURCE_OFFSET, ENHANCED_MAX_DISTANCE, ENHANCED_RADIUS, 0.0),
          aim: Vec4::new(aim.x, aim.y, 0.0, 0.0),
          streak: Vec4::new(ENHANCED_LENGTH, ENHANCED_WIDTH, SPEED_MIN * speed, SPEED_MAX * speed),
          look: ENHANCED_LOOK,
          splash_look: ENHANCED_SPLASH_LOOK,
          count: Self::get_enhanced_count(density),
          is_enhanced: 1,
          ..base
        }
      }
    }
  }

  /// Streaks drawn at a density, `0.5 * (1 + factor) * max_desired_items`.
  pub fn get_count(density: f32) -> u32 {
    (0.5 * (1.0 + density.clamp(0.0, 1.0)) * Self::STREAKS as f32) as u32
  }

  /// The same rain drawn over an image `size` texels across and down, which the enhanced streaks place themselves on.
  pub fn over(self, (width, height): (u32, u32)) -> Self {
    Self {
      aim: Vec4::new(self.aim.x, self.aim.y, width as f32, height as f32),
      ..self
    }
  }

  /// Where the enhanced drops' square stands from the eye: `aim` from last frame, eased `step` seconds towards
  /// Monolith's `Born` aim along `look`, the camera's way (`mad(view, dir, 5)`, its height dropped); there at once
  /// without one.
  pub fn ease_aim(aim: Option<Vec2>, look: Vec3, step: f32) -> Vec2 {
    let target: Vec2 = Vec2::new(look.x, look.z) * ENHANCED_AIM;

    aim.map_or(target, |aim| {
      aim.lerp(target, 1.0 - (-step / ENHANCED_AIM_EASING).exp())
    })
  }

  /// Enhanced drops drawn at a density, Monolith's `0.01 * (1 + factor * 99) * rain_max_particles`.
  pub fn get_enhanced_count(density: f32) -> u32 {
    (0.01 * (1.0 + density.clamp(0.0, 1.0) * 99.0) * ENHANCED_DROPS) as u32
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
