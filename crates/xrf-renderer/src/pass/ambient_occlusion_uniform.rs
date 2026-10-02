use glam::Mat4;

use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;

/// XeGTAO's `FinalValuePower`, the curve a strength of one gives.
const FINAL_POWER: f32 = 2.2;

/// XeGTAO's `RadiusMultiplier`: what the radius asked is widened by, against screen space's own bias to less.
const RADIUS_MULTIPLIER: f32 = 1.457;

/// The share of the search target's height a horizon is searched across at most, however near the point.
const MAX_REACH: f32 = 0.25;

/// What the occlusion's search reads, as `shaders/frame/ambient_occlusion.wgsl` declares it.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AmbientOcclusionUniform {
  pub radius: f32,
  pub power: f32,
  /// Metres one pixel of the search target spans at a metre from the camera.
  pub spread: f32,
  /// Pixels of the search target a horizon is searched across at most.
  pub reach: f32,
}

impl AmbientOcclusionUniform {
  /// The settings over a search target of a size, through the camera's projection.
  pub fn new(settings: &RenderAmbientOcclusionSettings, projection: Mat4, (width, height): (u32, u32)) -> Self {
    Self {
      radius: settings.radius * RADIUS_MULTIPLIER,
      power: FINAL_POWER * settings.strength,
      // Clip x over view x at a metre is the projection's first element.
      spread: 2.0 / (projection.x_axis.x * width.max(1) as f32),
      reach: (height as f32 * MAX_REACH).max(1.0),
    }
  }
}
