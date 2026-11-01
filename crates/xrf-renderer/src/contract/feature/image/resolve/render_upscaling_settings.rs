use serde::{Deserialize, Serialize};
use xrf_renderer_derive::RenderSettingSchema;

use crate::contract::feature::image::resolve::render_dlss_preset::RenderDlssPreset;
use crate::contract::feature::image::resolve::render_ray_reconstruction_preset::RenderRayReconstructionPreset;
use crate::contract::view::output::render_scale::RenderScale;

/// What the scene is drawn at, a share of the viewport upscaled to it, how sharply the upscaled frame is finished,
/// which model DLSS upscales with, whether DLSS reconstructs rays as it upscales, and with which model.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, RenderSettingSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderUpscalingSettings {
  pub scale: RenderScale,
  /// RCAS's sharpness while upscaled or resolved temporally, from none to its most.
  #[setting(min = 0, max = 1, recommended = [0, 0.5])]
  pub sharpening: f32,
  pub dlss_preset: RenderDlssPreset,
  /// DLSS ray reconstruction in super resolution's place: one network denoising and upscaling the frame over its
  /// surfaces' guides, where the GPU has it and the scene holds linear light.
  pub is_ray_reconstructed: bool,
  pub ray_reconstruction_preset: RenderRayReconstructionPreset,
}

impl Default for RenderUpscalingSettings {
  /// Drawn at the viewport's size; sharpened a tenth once upscaled; DLSS's models NVIDIA's pick, reconstructing no
  /// rays.
  fn default() -> Self {
    Self {
      scale: RenderScale::Native,
      sharpening: 0.1,
      dlss_preset: RenderDlssPreset::Default,
      is_ray_reconstructed: false,
      ray_reconstruction_preset: RenderRayReconstructionPreset::Default,
    }
  }
}

impl RenderUpscalingSettings {
  /// RCAS's `con`: FSR's remap of a sharpness to stops, `(1 - sharpening) * 2`, as `exp2(-stops)`.
  pub fn get_sharpness(&self) -> f32 {
    (-(1.0 - self.sharpening.clamp(0.0, 1.0)) * 2.0).exp2()
  }
}
