/// How the passes that turn their samples per pixel move each pixel's noise from one frame to the next, cycling over
/// a number of frames: by the R2 sequence's steps, or by interleaved gradient noise's.
pub struct NoiseTurn;

impl NoiseTurn {
  /// The R2 sequence's steps (Roberts, 2018): the plastic number's reciprocal and its square, as `common/noise`'s
  /// `R2_STEPS`.
  pub const R2_STEPS: [f32; 2] = [0.754_877_7, 0.569_840_3];

  /// Interleaved gradient noise's step between frames, Jimenez's, in pixels.
  pub const GRADIENT_STEP: f32 = 5.588_238;

  /// Frames the noise cycles through before it repeats.
  pub const FRAMES: u32 = 64;

  /// What `frame` turns a pixel's noise by each way, from zero to one, cycling every `frames`.
  pub fn get_r2(frame: u32, frames: u32) -> [f32; 2] {
    let turn: f32 = (frame % frames) as f32;

    [(turn * Self::R2_STEPS[0]).fract(), (turn * Self::R2_STEPS[1]).fract()]
  }

  /// The pixels `frame` moves interleaved gradient noise by, cycling every `FRAMES`.
  pub fn get_gradient(frame: u32) -> f32 {
    (frame % Self::FRAMES) as f32 * Self::GRADIENT_STEP
  }
}
