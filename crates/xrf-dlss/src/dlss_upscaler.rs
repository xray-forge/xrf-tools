#[cfg(feature = "sdk")]
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_error::XrfResult;

use crate::dlss_inputs::DlssInputs;
use crate::dlss_preset::DlssPreset;
use crate::dlss_quality::DlssQuality;
#[cfg(feature = "sdk")]
use crate::ngx::{ngx_sdk::NgxSdk, ngx_super_resolution::NgxSuperResolution};

/// One view's DLSS super resolution: what it upscales into, at which quality and preset, and the render sizes it
/// upscales from.
pub struct DlssUpscaler {
  output: (u32, u32),
  quality: DlssQuality,
  preset: DlssPreset,
  is_hdr: bool,
  #[cfg(feature = "sdk")]
  feature: Mutex<NgxSuperResolution>,
  #[cfg(feature = "sdk")]
  adapter: wgpu::Adapter,
  #[cfg(not(feature = "sdk"))]
  never: std::convert::Infallible,
}

impl DlssUpscaler {
  /// Whether it upscales into `output` pixels at a quality and preset over colour of linear light where `is_hdr`,
  /// rather than another one being needed.
  pub fn is_fitting(&self, output: (u32, u32), (quality, preset): (DlssQuality, DlssPreset), is_hdr: bool) -> bool {
    self.output == output && self.quality == quality && self.preset == preset && self.is_hdr == is_hdr
  }

  /// The smallest and the largest render size it upscales from.
  pub fn get_render_range(&self) -> ((u32, u32), (u32, u32)) {
    #[cfg(feature = "sdk")]
    {
      self.lock().render_range
    }

    #[cfg(not(feature = "sdk"))]
    match self.never {}
  }

  /// Records the upscale of `inputs`: their barriers into `encoder`, and NGX's work into the command buffer answered,
  /// which is submitted right after `encoder`'s commands and before any recorded after them.
  ///
  /// # Errors
  ///
  /// Returns NGX's error where it refuses the inputs.
  pub fn record(&self, encoder: &mut wgpu::CommandEncoder, inputs: &DlssInputs<'_>) -> XrfResult<wgpu::CommandBuffer> {
    #[cfg(feature = "sdk")]
    {
      self.lock().record(encoder, &self.adapter, inputs)
    }

    #[cfg(not(feature = "sdk"))]
    {
      let _ = (encoder, inputs);

      match self.never {}
    }
  }

  /// NGX's super resolution for `output` pixels at a quality and preset: reversed depth and motion at the render
  /// size, exposed by NGX itself over colour of linear light.
  #[cfg(feature = "sdk")]
  pub(crate) fn new(
    (sdk, adapter): (&Arc<Mutex<NgxSdk>>, &wgpu::Adapter),
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    (output, quality, preset, is_hdr): ((u32, u32), DlssQuality, DlssPreset, bool),
  ) -> XrfResult<Self> {
    use crate::ngx::bindings::*;

    let mut flags: NVSDK_NGX_DLSS_Feature_Flags =
      NVSDK_NGX_DLSS_Feature_Flags_NVSDK_NGX_DLSS_Feature_Flags_DepthInverted
        | NVSDK_NGX_DLSS_Feature_Flags_NVSDK_NGX_DLSS_Feature_Flags_MVLowRes;

    if is_hdr {
      flags |= NVSDK_NGX_DLSS_Feature_Flags_NVSDK_NGX_DLSS_Feature_Flags_IsHDR
        | NVSDK_NGX_DLSS_Feature_Flags_NVSDK_NGX_DLSS_Feature_Flags_AutoExposure;
    }

    let feature: NgxSuperResolution = NgxSuperResolution::new(sdk, (device, queue), (output, quality, preset), flags)?;

    Ok(Self {
      output,
      quality,
      preset,
      is_hdr,
      feature: Mutex::new(feature),
      adapter: adapter.clone(),
    })
  }

  #[cfg(feature = "sdk")]
  fn lock(&self) -> MutexGuard<'_, NgxSuperResolution> {
    self.feature.lock().unwrap_or_else(PoisonError::into_inner)
  }
}
