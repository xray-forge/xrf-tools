use std::ptr;

use wgpu::hal::api::Vulkan;
use xrf_error::{XrfError, XrfResult};

use crate::ngx::bindings::*;
use crate::ngx::ngx_application::NgxApplication;
use crate::ngx::ngx_result::check_ngx;

/// NGX started on a Vulkan device, and the parameters its features are made with.
pub(crate) struct NgxSdk {
  pub(crate) parameters: *mut NVSDK_NGX_Parameter,
  pub(crate) device: wgpu::Device,
}

// SAFETY: NGX's parameters are only touched under the mutex `Dlss` keeps the SDK in.
unsafe impl Send for NgxSdk {}
unsafe impl Sync for NgxSdk {}

impl NgxSdk {
  /// Starts NGX on `device`, a Vulkan device opened with its extensions, where it offers super resolution; asks the
  /// driver to update the runtime library meanwhile.
  pub(crate) fn start(application: &NgxApplication, device: &wgpu::Device) -> XrfResult<Self> {
    Self::update(application);

    unsafe {
      let hal = device
        .as_hal::<Vulkan>()
        .ok_or_else(|| XrfError::new_unexpected_error("DLSS needs a Vulkan device"))?;
      let shared = hal.shared_instance();
      let instance = shared.raw_instance();
      let engine_version = application.get_engine_version();

      application.with_feature_info(Default::default(), |info| {
        check_ngx(
          NVSDK_NGX_VULKAN_Init_with_ProjectID(
            application.get_project().as_ptr(),
            NVSDK_NGX_EngineType_NVSDK_NGX_ENGINE_TYPE_CUSTOM,
            engine_version.as_ptr(),
            info.ApplicationDataPath,
            instance.handle(),
            hal.raw_physical_device(),
            hal.raw_device().handle(),
            shared.entry().static_fn().get_instance_proc_addr,
            instance.fp_v1_0().get_device_proc_addr,
            info.FeatureInfo,
            NVSDK_NGX_Version_NVSDK_NGX_Version_API,
          ),
          "NGX does not start",
        )
      })?;
    }

    // Owned from here, so a refusal below shuts down what was started.
    let mut sdk: Self = Self {
      parameters: ptr::null_mut(),
      device: device.clone(),
    };
    let mut is_available: i32 = 0;

    check_ngx(
      unsafe { NVSDK_NGX_VULKAN_GetCapabilityParameters(&mut sdk.parameters) },
      "NGX has no parameters",
    )?;

    check_ngx(
      unsafe {
        NVSDK_NGX_Parameter_GetI(
          sdk.parameters,
          NVSDK_NGX_Parameter_SuperSampling_Available.as_ptr().cast(),
          &mut is_available,
        )
      },
      "NGX does not say whether it offers super resolution",
    )?;

    if is_available == 0 {
      return Err(XrfError::new_unexpected_error(
        "NGX offers no super resolution on this GPU or driver",
      ));
    }

    Ok(sdk)
  }

  /// Asks the driver, on a thread of its own, to fetch a newer super resolution library where NVIDIA offers one.
  fn update(application: &NgxApplication) {
    let application: NgxApplication = application.clone();

    std::thread::spawn(move || {
      application.with_feature_info(NVSDK_NGX_Feature_NVSDK_NGX_Feature_SuperSampling, |info| unsafe {
        NVSDK_NGX_UpdateFeature(&info.Identifier, info.FeatureID);
      });
    });
  }
}

impl Drop for NgxSdk {
  /// Waits for what was submitted to finish, through wgpu, which owns the queue, then releases the parameters, where it
  /// was given them, and shuts NGX down on the device.
  fn drop(&mut self) {
    if let Err(error) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
      log::error!("DLSS shuts down on a device that does not finish its work: {error}");
    }

    unsafe {
      let Some(hal) = self.device.as_hal::<Vulkan>() else {
        return;
      };

      if !self.parameters.is_null()
        && let Err(error) = check_ngx(
          NVSDK_NGX_VULKAN_DestroyParameters(self.parameters),
          "NGX's parameters are not released",
        )
      {
        log::error!("{error}");
      }

      if let Err(error) = check_ngx(
        NVSDK_NGX_VULKAN_Shutdown1(hal.raw_device().handle()),
        "NGX does not shut down",
      ) {
        log::error!("{error}");
      }
    }
  }
}
