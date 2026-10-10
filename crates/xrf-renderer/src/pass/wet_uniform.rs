use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// What `shaders/common/wet.wgsl` declares as `Wet`: how hard it rains, the clock, the engine, the cover, and the
/// enhanced wetting's state.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "Wet")]
pub struct WetUniform {
  /// `RainDensity.x`, the falling rain's, which splashes and ripples by; none once it stops.
  pub density: f32,
  /// Seconds the rain has fallen.
  pub time: f32,
  /// One on Anomaly's engine.
  pub is_extended: f32,
  /// One where the surfaces are wetted as the enhanced rain wets them.
  pub is_enhanced: f32,
  /// The rain cover's centre in `x` and `z`, its half width, and the height it is seen from; then its texels across,
  /// the metres it reaches down, the metres the wetness creeps under a cover from its wet edges, and the pixels the
  /// creep's noise is moved by this frame.
  pub cover: Vec4,
  pub cover_state: Vec4,
  /// The level's surface map's centre in `x` and `z`, its half width, and the height it is seen from; then its texels
  /// across and the metres it reaches down.
  pub surface: Vec4,
  pub surface_shape: Vec4,
  /// The level's wetness, how much of flat terrain puddles may cover, how much of a reflection a puddle takes at most,
  /// and how strongly rain ripples.
  pub puddles: Vec4,
  /// Metres out to which puddles are drawn, how full they are, how stormy the rain has been, and how soaked a long
  /// rain has left the level.
  pub puddle_state: Vec4,
  /// One where the reflections are traced, so the wetness wets the gloss as far as the rain reaches for them to
  /// trace, rather than over the metres the engine fades its splashes over; then the metres it reaches; the rest
  /// unused.
  pub reach: Vec4,
  /// The way and speed gusts drift across the puddles, metres a second in renderer `x` and `z`; the rest unused.
  pub gust: Vec4,
}

impl WetUniform {
  /// Metres a second gusts drift at for a keyframe's `wind_velocity`, a hundredth of it, and the least and most.
  const GUST_SPEED: (f32, f32, f32) = (0.01, 1.0, 12.0);

  /// Metres the wetness creeps under a cover from its wet edges once a long rain has soaked the level through.
  pub const CREEP_MOST: f32 = 1.5;
  /// Interleaved gradient noise's step between frames, Jimenez's, in pixels, and the frames it cycles through.
  const NOISE_STEP: f32 = 5.588_238;
  const NOISE_FRAMES: u32 = 64;

  /// The way and speed gusts drift for a wind: downwind, as the rain leans, in renderer space.
  pub fn get_gust(direction: f32, velocity: f32) -> Vec4 {
    let (scale, least, most): (f32, f32, f32) = Self::GUST_SPEED;
    let speed: f32 = (velocity * scale).clamp(least, most);

    Vec4::new(-direction.sin() * speed, -direction.cos() * speed, 0.0, 0.0)
  }

  /// The pixels the cover's noise is moved by on the noise's `frame`.
  pub fn get_cover_noise(frame: u32) -> f32 {
    (frame % Self::NOISE_FRAMES) as f32 * Self::NOISE_STEP
  }
}
