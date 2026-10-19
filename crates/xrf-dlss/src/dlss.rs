#[cfg(feature = "sdk")]
use std::sync::{Arc, Mutex};

use xrf_error::XrfResult;

use crate::dlss_preset::DlssPreset;
use crate::dlss_quality::DlssQuality;
use crate::dlss_ray_preset::DlssRayPreset;
use crate::dlss_ray_reconstructor::DlssRayReconstructor;
use crate::dlss_upscaler::DlssUpscaler;
#[cfg(feature = "sdk")]
use crate::ngx::{ngx_application::NgxApplication, ngx_sdk::NgxSdk};

/// DLSS started on a device: NGX, which each view makes its upscaler or ray reconstructor from.
#[derive(Clone)]
pub struct Dlss {
  #[cfg(feature = "sdk")]
  sdk: Arc<Mutex<NgxSdk>>,
  #[cfg(feature = "sdk")]
  adapter: wgpu::Adapter,
  #[cfg(feature = "sdk")]
  device: wgpu::Device,
  #[cfg(feature = "sdk")]
  queue: wgpu::Queue,
  #[cfg(not(feature = "sdk"))]
  never: std::convert::Infallible,
}

impl Dlss {
  /// Whether the build has DLSS, built against its SDK.
  pub const IS_BUILT: bool = cfg!(feature = "sdk");

  /// The file name of the runtime library NGX loads super resolution from, which an application ships, ray
  /// reconstruction's beside it.
  pub const RUNTIME_LIBRARY: &'static str = if cfg!(target_os = "windows") {
    "nvngx_dlss.dll"
  } else {
    "libnvidia-ngx-dlss.so.310.9.1"
  };

  /// NGX on a device: a Vulkan one opened with its instance and device extensions, or a D3D12 one; none where the GPU
  /// or driver has no DLSS.
  #[cfg(feature = "sdk")]
  pub(crate) fn start(
    application: &NgxApplication,
    adapter: &wgpu::Adapter,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
  ) -> Option<Self> {
    match NgxSdk::start(application, device) {
      Ok(sdk) => Some(Self {
        sdk: Arc::new(Mutex::new(sdk)),
        adapter: adapter.clone(),
        device: device.clone(),
        queue: queue.clone(),
      }),
      Err(error) => {
        log::info!("DLSS is unavailable: {error}");

        None
      }
    }
  }

  /// Whether the GPU and its driver reconstruct rays as well as they upscale.
  pub fn has_ray_reconstruction(&self) -> bool {
    #[cfg(feature = "sdk")]
    {
      self
        .sdk
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .has_ray_reconstruction
    }

    #[cfg(not(feature = "sdk"))]
    match self.never {}
  }

  /// A ray reconstructor of one view into `output` pixels at a quality and preset, over colour of linear light already
  /// exposed.
  ///
  /// # Errors
  ///
  /// Returns NGX's error where the reconstructor cannot be made.
  pub fn create_ray_reconstructor(
    &self,
    output: (u32, u32),
    (quality, preset): (DlssQuality, DlssRayPreset),
  ) -> XrfResult<DlssRayReconstructor> {
    #[cfg(feature = "sdk")]
    {
      DlssRayReconstructor::new(
        (&self.sdk, &self.adapter),
        (&self.device, &self.queue),
        (output, quality, preset),
      )
    }

    #[cfg(not(feature = "sdk"))]
    {
      let _ = (output, quality, preset);

      match self.never {}
    }
  }

  /// An upscaler of one view into `output` pixels at a quality and preset, over colour of linear light where
  /// `is_hdr`.
  ///
  /// # Errors
  ///
  /// Returns NGX's error where the upscaler cannot be made.
  pub fn create_upscaler(
    &self,
    output: (u32, u32),
    (quality, preset): (DlssQuality, DlssPreset),
    is_hdr: bool,
  ) -> XrfResult<DlssUpscaler> {
    #[cfg(feature = "sdk")]
    {
      DlssUpscaler::new(
        (&self.sdk, &self.adapter),
        (&self.device, &self.queue),
        (output, quality, preset, is_hdr),
      )
    }

    #[cfg(not(feature = "sdk"))]
    {
      let _ = (output, quality, preset, is_hdr);

      match self.never {}
    }
  }
}
