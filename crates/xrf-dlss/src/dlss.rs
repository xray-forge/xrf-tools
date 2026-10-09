#[cfg(feature = "sdk")]
use std::sync::{Arc, Mutex};

use xrf_error::XrfResult;

use crate::dlss_preset::DlssPreset;
use crate::dlss_quality::DlssQuality;
use crate::dlss_upscaler::DlssUpscaler;
#[cfg(feature = "sdk")]
use crate::ngx::{ngx_application::NgxApplication, ngx_sdk::NgxSdk};

/// DLSS started on a device: NGX, which each view makes its upscaler from.
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
  /// The file name of the runtime library NGX loads super resolution from, which an application ships.
  pub const RUNTIME_LIBRARY: &'static str = if cfg!(target_os = "windows") {
    "nvngx_dlss.dll"
  } else {
    "libnvidia-ngx-dlss.so.310.9.1"
  };

  /// NGX on a device opened with its instance and device extensions; none where the GPU or driver has no DLSS.
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
