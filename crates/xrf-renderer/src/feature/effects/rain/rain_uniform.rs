use xrf_math::glam::{Vec3, Vec4};
use xrf_renderer_core::ShaderStruct;

use crate::contract::feature::effects::render_rain_drops_mode::RenderRainDropsMode;
use crate::feature::effects::rain::rain_fall::RainFall;
use crate::feature::effects::rain::rain_frame::RainFrame;

/// `drop_length` and `drop_width`, which is half a streak's width.
const STREAK_LENGTH: f32 = 5.0;
const STREAK_WIDTH: f32 = 0.3;

/// The narrowest and widest square the enhanced drops fall over, in metres.
const ENHANCED_SQUARE: (f32, f32) = (5.0, 150.0);

/// The most rings of far cells about the near square, as `frame/rain.wgsl` walks them (`FAR_RINGS_MOST`).
const FAR_RINGS_MOST: u32 = 8;

/// The least of their designed size the enhanced drops' rings are drawn at, their waves a few of the rings' texels.
const RING_SIZE_LEAST: f32 = 0.5;

/// `drop_speed_min` and `drop_speed_max`, in metres a second.
const SPEED_MIN: f32 = 40.0;
const SPEED_MAX: f32 = 80.0;

/// `ssfx_rain_drops_setup`: drops at the heaviest. Each of its 2500 streaks carries two drops, where each of ours
/// carries one, falling on its own; and Monolith's drops are born again as they land, where a stateless fall waits out
/// its turn, about a quarter of it: so 6000 for its 5000 drops.
const ENHANCED_DROPS: f32 = 6000.0;

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

/// The enhanced splash: how much the light it scatters covers, how far its map bends the ground through the distortion
/// target (which the window pass scales by `def_distort`), unused, and the share of the landings that splash.
const ENHANCED_SPLASH_LOOK: Vec4 = Vec4::new(0.08, 0.5, 0.0, ENHANCED_SPLASHING);

/// What `shaders/frame/rain.wgsl` reads as its `Rain`, as `dxRainRender::Render` sets the rain up.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Rain")]
pub(crate) struct RainUniform {
  /// `rain_color`, then `factor / 2 + .5`, which the streaks are drawn long by and the engine's covered by.
  pub color: Vec4,
  /// The way the rain falls, in renderer space, then the widest a drop strays from it, in radians, as Monolith's
  /// `random_dir` call scatters it.
  pub axis: Vec4,
  /// The cover's centre in `x` and `z`, its half width, and the height it is seen from; then its texels across and
  /// the metres it reaches down, the rest unused.
  pub window: Vec4,
  pub cover_shape: Vec4,
  /// The level water's map's window, laid out as the cover's, and the shape it shares with the level surface's, which
  /// the enhanced drops land on; none without it and for the engine's streaks.
  pub water: Vec4,
  pub water_shape: Vec4,
  /// The level surface's map's window, whose lowest heights tell an enhanced drop landing on foliage; none without it.
  pub surface: Vec4,
  /// Metres over the eye a drop starts, metres it falls at most, the side of the square it falls over, which its
  /// columns repeat over, and the enhanced drops' rings' size, of their designed one.
  pub fall: Vec4,
  /// A drop's length at the heaviest rain and half its width, then its slowest and fastest speed in metres a second.
  pub streak: Vec4,
  /// The enhanced drops' cover, the share of the light they scatter, refraction and reflection; then the splashes'
  /// cover of the light they scatter, bend of the ground, unused, and share of the landings that splash.
  pub look: Vec4,
  pub splash_look: Vec4,
  /// Where the drops' square stands from the eye, in `x` and `z`, then the texels across and down of the image the
  /// enhanced streaks draw over.
  pub aim: Vec4,
  /// How the frame is shown (`DisplayMode::get_display`), which the streaks and the engine's splashes, pictures painted
  /// for the engine's display, enter the scene through; the engine's pipelines bind no lighting to read it from.
  pub display: Vec4,
  /// Drops drawn.
  pub count: u32,
  /// The rain's clock in seconds, run at its pace and wrapped within `CLOCK_PERIOD`.
  pub time: f32,
  /// Indices the splash's model draws.
  pub splash_indices: u32,
  /// One where the drops are the enhanced ones, by their look and where they land.
  pub is_enhanced: u32,
  /// How fast the rain's clock runs against real time: the fall's pace by the wind (`RainFall::pace`).
  pub pace: f32,
  /// Metres across the near square every drop falls in, which the enhanced drops' rings wrap over.
  pub near: f32,
  /// The drops in the near square, the first of them, which alone ring and splash; the rest fall in far cells.
  pub near_count: u32,
  pub _pad: f32,
}

impl RainUniform {
  /// `max_desired_items`: streaks at the heaviest rain; the lightest's by the fall (`RainFall::share`).
  pub(crate) const STREAKS: u32 = 2500;
  /// Seconds the rain's clock repeats over; the wet surfaces' splashes, flows and ripples each repeat a whole number of
  /// times in it (`WetUniform::time`). Each drop starts a new fall at the wrap, as it does every second or two anyway.
  pub(crate) const CLOCK_PERIOD: f64 = 2000.0;

  /// One `frame`'s rain: falling as its `RainFall` says, drawn as its drops are.
  pub(crate) fn new(frame: RainFrame<'_>) -> Self {
    let fall: RainFall = RainFall::new(&frame);
    let RainFrame {
      rain,
      drops,
      window,
      cover_shape,
      splash_indices,
      ..
    } = frame;
    let density: f32 = rain.density.clamp(0.0, 1.0);
    let base: Self = Self {
      color: rain.color.clamp(Vec3::ZERO, Vec3::ONE).extend(density / 2.0 + 0.5),
      axis: fall.axis.extend(fall.stray),
      window,
      cover_shape,
      fall: Vec4::new(fall.source_offset, fall.max_distance, fall.side, 0.0),
      aim: fall.aim.extend(0.0).extend(0.0),
      splash_indices,
      pace: fall.pace,
      near: fall.side,
      ..Default::default()
    };

    match drops {
      RenderRainDropsMode::Engine => {
        let count: u32 = fall.get_count(Self::STREAKS as f32);

        Self {
          streak: Vec4::new(STREAK_LENGTH, STREAK_WIDTH, SPEED_MIN, SPEED_MAX),
          count,
          near_count: count,
          ..base
        }
      }
      RenderRainDropsMode::Enhanced => {
        let count: u32 = fall.get_count(ENHANCED_DROPS);

        Self {
          streak: Vec4::new(
            ENHANCED_LENGTH,
            ENHANCED_WIDTH,
            SPEED_MIN * ENHANCED_SPEED,
            SPEED_MAX * ENHANCED_SPEED,
          ),
          look: ENHANCED_LOOK,
          splash_look: ENHANCED_SPLASH_LOOK,
          count,
          near_count: count,
          is_enhanced: 1,
          ..base
        }
      }
    }
  }

  /// Metres a side of the tile the enhanced drops' rings wrap over; none for the engine's streaks or without the level
  /// water's map to land on.
  pub(crate) fn get_rings(&self) -> f32 {
    match self.is_enhanced != 0 && self.water.z > 0.0 {
      false => 0.0,
      true => self.near,
    }
  }

  /// The same rain, its enhanced drops falling over a square `side` metres across as `frame/rain.wgsl`'s `far_drop`
  /// places them; the engine's streaks as they are.
  pub(crate) fn squared(self, side: f32) -> Self {
    if self.is_enhanced == 0 {
      return self;
    }

    let side: f32 = side.clamp(ENHANCED_SQUARE.0, ENHANCED_SQUARE.1);
    let near: f32 = side.min(RainFall::SIDE);
    let near_count: u32 = (self.count as f32 * (near / RainFall::SIDE).powi(2)).round() as u32;
    let rings: u32 = ((side - near) / (2.0 * near)).ceil() as u32;

    debug_assert!(rings <= FAR_RINGS_MOST, "{rings} rings of far cells");

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
  pub(crate) fn ringed(self, size: f32) -> Self {
    Self {
      fall: self.fall.truncate().extend(size.max(RING_SIZE_LEAST)),
      ..self
    }
  }

  /// The same rain drawn over an image `size` texels across and down, which the enhanced streaks place themselves on.
  pub(crate) fn over(self, (width, height): (u32, u32)) -> Self {
    Self {
      aim: Vec4::new(self.aim.x, self.aim.y, width as f32, height as f32),
      ..self
    }
  }

  /// The same rain on its clock `time`, at its `pace` and leaning along `axis`, as eased from frame to frame.
  pub(crate) fn clocked(self, time: f32, pace: f32, axis: Vec3) -> Self {
    Self {
      time,
      pace,
      axis: axis.extend(self.axis.w),
      ..self
    }
  }
}
