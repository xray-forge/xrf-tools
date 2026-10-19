#[cfg(feature = "sdk")]
use std::sync::{Arc, Mutex};

use xrf_error::XrfResult;

use crate::dlss_quality::DlssQuality;
use crate::dlss_ray_inputs::DlssRayInputs;
use crate::dlss_ray_preset::DlssRayPreset;
#[cfg(feature = "sdk")]
use crate::ngx::{ngx_ray_reconstruction::NgxRayReconstruction, ngx_sdk::NgxSdk};

/// One view's DLSS ray reconstruction: what it denoises and upscales into, at which quality, and the render sizes it
/// upscales from.
pub struct DlssRayReconstructor {
  output: (u32, u32),
  quality: DlssQuality,
  preset: DlssRayPreset,
  #[cfg(feature = "sdk")]
  feature: NgxRayReconstruction,
  #[cfg(feature = "sdk")]
  adapter: wgpu::Adapter,
  #[cfg(not(feature = "sdk"))]
  never: std::convert::Infallible,
}

impl DlssRayReconstructor {
  /// Whether it reconstructs into `output` pixels at a quality and preset, rather than another one being needed.
  pub fn is_fitting(&self, output: (u32, u32), (quality, preset): (DlssQuality, DlssRayPreset)) -> bool {
    self.output == output && self.quality == quality && self.preset == preset
  }

  /// The smallest and the largest render size it upscales from.
  pub fn get_render_range(&self) -> ((u32, u32), (u32, u32)) {
    #[cfg(feature = "sdk")]
    {
      self.feature.render_range
    }

    #[cfg(not(feature = "sdk"))]
    match self.never {}
  }

  /// Records the reconstruction of `inputs`: their barriers into `encoder`, and NGX's work into the command buffer
  /// answered, which is submitted right after `encoder`'s commands and before any recorded after them.
  ///
  /// # Errors
  ///
  /// Returns NGX's error where it refuses the inputs.
  pub fn record(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    inputs: &DlssRayInputs<'_>,
  ) -> XrfResult<wgpu::CommandBuffer> {
    #[cfg(feature = "sdk")]
    {
      self.feature.record(encoder, &self.adapter, inputs)
    }

    #[cfg(not(feature = "sdk"))]
    {
      let _ = (encoder, inputs);

      match self.never {}
    }
  }

  /// NGX's ray reconstruction for `output` pixels at a quality and preset: reversed hardware depth and motion at the
  /// render size, over colour of linear light already exposed, as ray reconstruction exposes nothing itself.
  #[cfg(feature = "sdk")]
  pub(crate) fn new(
    (sdk, adapter): (&Arc<Mutex<NgxSdk>>, &wgpu::Adapter),
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    (output, quality, preset): ((u32, u32), DlssQuality, DlssRayPreset),
  ) -> XrfResult<Self> {
    use crate::ngx::bindings::*;

    let flags: NVSDK_NGX_DLSS_Feature_Flags = NVSDK_NGX_DLSS_Feature_Flags_NVSDK_NGX_DLSS_Feature_Flags_DepthInverted
      | NVSDK_NGX_DLSS_Feature_Flags_NVSDK_NGX_DLSS_Feature_Flags_MVLowRes
      | NVSDK_NGX_DLSS_Feature_Flags_NVSDK_NGX_DLSS_Feature_Flags_IsHDR;
    let feature: NgxRayReconstruction =
      NgxRayReconstruction::new(sdk, (device, queue), (output, quality, preset), flags)?;

    Ok(Self {
      output,
      quality,
      preset,
      feature,
      adapter: adapter.clone(),
    })
  }
}
