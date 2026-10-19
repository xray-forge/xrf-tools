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

/// The narrowest and widest square the enhanced drops fall over, in metres: past Monolith's they thin out with
/// distance (`squared`), so the widest falls under seven times as many drops.
const ENHANCED_SQUARE: (f32, f32) = (5.0, 150.0);

/// The least of their designed size the enhanced drops' rings are drawn at, where their waves still span a few of the
/// rings' texels.
const RING_SIZE_LEAST: f32 = 0.5;

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
  /// The level water's map's window, laid out as the cover's, and its texels across and the metres it reaches down,
  /// which the enhanced drops land on; none without it, or for the engine's streaks, which fall through the water as
  /// the engine's do.
  pub water: Vec4,
  pub water_shape: Vec4,
  /// Metres over the eye a drop starts, metres it falls at most, the side of the square it falls over, which its
  /// columns repeat over, and the enhanced drops' rings' size, of their designed one.
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
  /// The rain's clock: seconds it has fallen, run at its pace.
  pub time: f32,
  /// Indices the splash's model draws.
  pub splash_indices: u32,
  /// One where the drops are the enhanced ones, straying as Monolith's `random_dir` scatters them.
  pub is_enhanced: u32,
  /// How fast the rain's clock runs against real time: the enhanced drops' speed, of their fastest, which Monolith
  /// scales by the wind; one for the engine's.
  pub pace: f32,
  /// Metres across the square about the enhanced drops' centre every drop falls in, past which they thin out with
  /// distance, and which their rings wrap over: Monolith's, or the whole square where it is narrower.
  pub near: f32,
  /// The drops falling in the near square, the first of them; the rest fall far, fewer a cell the further out
  /// (`squared`), and only the near ones ring and splash.
  pub near_count: u32,
  pub _pad: f32,
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
      pace: 1.0,
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
          near_count: Self::get_count(density),
          ..base
        }
      }
      RenderRainDropsMode::Enhanced => {
        // Monolith reads `wind_velocity` over a thousand; its noise swings the lean by up to twice its own two thirds,
        // where `wind_strength_factor` swings by one about a half.
        let velocity: f32 = (wind.velocity * 0.001).clamp(0.0, 1.0);
        let lean: f32 = (velocity + (strength - 0.5) * 2.0).clamp(0.0, 1.0);
        // Monolith's speed by the wind, which runs the rain's clock slower, so a drop never jumps along its fall as
        // the wind changes (`FallingRainFeature::prepare` runs it).
        let pace: f32 = (velocity * 1.5).clamp(0.5, 1.0);

        Self {
          axis: Self::get_axis(ENHANCED_MAX_LEAN * lean, wind.direction).extend(ENHANCED_STRAY * (1.5 - velocity)),
          fall: Vec4::new(ENHANCED_SOURCE_OFFSET, ENHANCED_MAX_DISTANCE, ENHANCED_RADIUS, 0.0),
          aim: Vec4::new(aim.x, aim.y, 0.0, 0.0),
          streak: Vec4::new(
            ENHANCED_LENGTH,
            ENHANCED_WIDTH,
            SPEED_MIN * ENHANCED_SPEED,
            SPEED_MAX * ENHANCED_SPEED,
          ),
          pace,
          look: ENHANCED_LOOK,
          splash_look: ENHANCED_SPLASH_LOOK,
          count: Self::get_enhanced_count(density),
          near_count: Self::get_enhanced_count(density),
          is_enhanced: 1,
          near: ENHANCED_RADIUS,
          ..base
        }
      }
    }
  }

  /// Metres a side of the tile the enhanced drops' rings wrap over, the square every drop falls in, as the water
  /// and the puddles read them (`common/rain_rings.wgsl`); none for the engine's streaks, or where no level water's
  /// map is drawn for the drops to land on (`water`), where the water and the puddles keep their maps' rings.
  pub fn get_rings(&self) -> f32 {
    match self.is_enhanced != 0 && self.water.z > 0.0 {
      false => 0.0,
      true => self.near,
    }
  }

  /// The same rain, its enhanced drops falling over a square `side` metres across; the engine's streaks as they are.
  /// Within Monolith's square every drop falls, a narrower one fewer by its area; past it the far drops fall in cells
  /// as wide, the one under its centre as many as it for what of it the square leaves out, and rings about it, a ring `k` cells out a quarter of the near square's drops over `k²` a cell
  /// (`frame/rain.wgsl`), so the drops on screen stay about as many at every distance, as a drop far off is drawn
  /// small: about `2 + 2 ln(side / 15)` times Monolith's in all.
  pub fn squared(self, side: f32) -> Self {
    if self.is_enhanced == 0 {
      return self;
    }

    let side: f32 = side.clamp(ENHANCED_SQUARE.0, ENHANCED_SQUARE.1);
    let near: f32 = side.min(ENHANCED_RADIUS);
    let near_count: u32 = (self.count as f32 * (near / ENHANCED_RADIUS).powi(2)).round() as u32;
    let rings: u32 = ((side - near) / (2.0 * near)).ceil() as u32;
    let far_count: u32 = (1..=rings)
      .map(|k| 8 * k * ((near_count as f32 / (4 * k * k) as f32 + 0.5).floor() as u32).max(1))
      .sum::<u32>()
      + if rings > 0 { near_count } else { 0 };

    Self {
      fall: Vec4::new(self.fall.x, self.fall.y, side, self.fall.w),
      count: near_count + far_count,
      near,
      near_count,
      ..self
    }
  }

  /// The same rain, its enhanced drops' rings `size` times their designed size, held off nought.
  pub fn ringed(self, size: f32) -> Self {
    Self {
      fall: self.fall.truncate().extend(size.max(RING_SIZE_LEAST)),
      ..self
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

    aim.map_or(target, |aim| aim.lerp(target, Self::get_following(step)))
  }

  /// The share of the way towards where it is going an eased value of the enhanced rain goes in `step` seconds: its
  /// drops' square, their pace and their lean, so a turn or a change of weather moves them over about a second.
  pub fn get_following(step: f32) -> f32 {
    1.0 - (-step / ENHANCED_AIM_EASING).exp()
  }

  /// The same rain on its clock `time`, at its `pace` and leaning along `axis`, as eased from frame to frame.
  pub fn clocked(self, (time, pace): (f32, f32), axis: Vec3) -> Self {
    Self {
      time,
      pace,
      axis: axis.extend(self.axis.w),
      ..self
    }
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
