use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

/// Frames VBAO accumulates over at most.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION: u32 = 32;

/// VBAO's search and strengths: the slices it turns about a pixel and the steps along each, how deep an occluder is
/// taken to be, how much light bounces back off a surface into its own creases, and over how many frames it is
/// gathered.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderAmbientOcclusionVbaoSettings {
  #[setting(min = 1, max = 4, recommended = [1, 2], wasted_above = 2)]
  pub slices: u32,
  #[setting(min = 2, max = 16, recommended = [4, 12], wasted_above = 12)]
  pub steps: u32,
  /// Metres behind what the depth shows that it is taken to be solid: light passes behind anything thinner.
  // The renderer's own: past two metres a thickness lets nothing through that a horizon would not.
  #[setting(min = 0.01, max = 2)]
  pub thickness: f32,
  /// How much of the light a surface's colour bounces between the sides of its creases comes back, one all of it.
  #[setting(min = 0, max = 1)]
  pub bounce: f32,
  /// Frames each pixel's occlusion is averaged over at most; one gathers none, its noise then holding still.
  #[setting(min = 1, max = RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION, recommended = [4, 16])]
  pub accumulation: u32,
}

impl Default for RenderAmbientOcclusionVbaoSettings {
  /// One slice of eight steps, a grass clump's depth, every bounce, and eight frames.
  fn default() -> Self {
    Self {
      slices: 1,
      steps: 8,
      thickness: 0.25,
      bounce: 1.0,
      accumulation: 8,
    }
  }
}

impl RenderAmbientOcclusionVbaoSettings {
  /// The least and most slices, and steps, it searches.
  pub const SLICES_RANGE: (u32, u32) = (1, 4);
  pub const STEPS_RANGE: (u32, u32) = (2, 16);

  pub fn get_slices(&self) -> u32 {
    self.slices.clamp(Self::SLICES_RANGE.0, Self::SLICES_RANGE.1)
  }

  pub fn get_steps(&self) -> u32 {
    self.steps.clamp(Self::STEPS_RANGE.0, Self::STEPS_RANGE.1)
  }

  /// Frames each pixel's occlusion is averaged over: one at least, and never past the most there are.
  pub fn get_accumulation(&self) -> u32 {
    self.accumulation.clamp(1, RENDER_MAX_AMBIENT_OCCLUSION_ACCUMULATION)
  }
}
