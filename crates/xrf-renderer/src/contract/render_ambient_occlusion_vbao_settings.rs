use serde::{Deserialize, Serialize};

/// Frames VBAO accumulates over at most.
pub const RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION: u32 = 32;

/// VBAO's strengths: how deep an occluder is taken to be, how much light bounces back off a
/// surface into its own creases, and over how many frames it is gathered.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAmbientOcclusionVbaoSettings {
  /// Metres behind what the depth shows that it is taken to be solid: light passes behind anything thinner.
  pub thickness: f32,
  /// How much of the light a surface's colour bounces between the sides of its creases comes back, one all of it.
  pub bounce: f32,
  /// Frames each pixel's occlusion is averaged over at most; one gathers none, its noise then holding still.
  pub accumulation: u32,
}

impl Default for RenderAmbientOcclusionVbaoSettings {
  /// A grass clump's depth, every bounce, and eight frames.
  fn default() -> Self {
    Self {
      thickness: 0.25,
      bounce: 1.0,
      accumulation: 8,
    }
  }
}

impl RenderAmbientOcclusionVbaoSettings {
  /// Frames each pixel's occlusion is averaged over: one at least, and never past the most there are.
  pub fn get_accumulation(&self) -> u32 {
    self.accumulation.clamp(1, RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION)
  }
}
