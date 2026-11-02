use std::ptr;
use std::sync::{MutexGuard, Once};
use std::time::Instant;

#[cfg(windows)]
use wgpu::hal::api::Dx12;
use wgpu::hal::api::Vulkan;
#[cfg(windows)]
use windows_core::Interface;
use xrf_error::{XrfError, XrfResult};

use crate::ngx::backend::ngx_api::NgxApi;
use crate::ngx::bindings::*;
use crate::ngx::feature::ngx_retired_feature::NgxRetiredFeature;
use crate::ngx::ngx_application::NgxApplication;
use crate::ngx::ngx_result::NgxResult;
use crate::ngx::sdk::ngx_lock::NgxLock;

/// NGX started on a device, the API it runs on, the parameters its features are made with, whether it reconstructs
/// rays as well as it upscales, and the features dropped while the device may still run their work.
pub(crate) struct NgxSdk {
  pub(crate) parameters: *mut NVSDK_NGX_Parameter,
  pub(crate) device: wgpu::Device,
  pub(crate) api: NgxApi,
  pub(crate) has_ray_reconstruction: bool,
  retired: Vec<NgxRetiredFeature>,
}

// SAFETY: the SDK is reached only through the mutex `Dlss` keeps it in, which only `Send` needs, and NGX's parameters
// only under that mutex and NGX's lock.
unsafe impl Send for NgxSdk {}

impl NgxSdk {
  /// Starts NGX on `device`, a Vulkan device opened with its extensions or a D3D12 one, where it offers super
  /// resolution; asks the driver to update the runtime library meanwhile.
  pub(crate) fn start(application: &NgxApplication, device: &wgpu::Device) -> XrfResult<Self> {
    let api: NgxApi =
      NgxApi::of(device).ok_or_else(|| XrfError::new_unexpected_error("DLSS needs a Vulkan or D3D12 device"))?;

    Self::update(application);

    let ngx: MutexGuard<'static, ()> = NgxLock::hold();

    match api {
      NgxApi::Vulkan => Self::init_vulkan(application, device)?,
      #[cfg(windows)]
      NgxApi::D3d12 => Self::init_d3d12(application, device)?,
    }

    // Owned from here, so a refusal below shuts down what was started: dropped once NGX's lock is let go, as its drop
    // takes the lock again.
    let mut sdk: Self = Self {
      parameters: ptr::null_mut(),
      device: device.clone(),
      api,
      has_ray_reconstruction: false,
      retired: Vec::new(),
    };
    let read: XrfResult<()> = sdk.read_capabilities();

    drop(ngx);
    read?;

    if !sdk.has_ray_reconstruction {
      log::info!("DLSS ray reconstruction is unavailable on this GPU or driver");
    }

    Ok(sdk)
  }

  /// Takes NGX's parameters and reads whether it upscales and reconstructs rays on the device, holding NGX's lock.
  fn read_capabilities(&mut self) -> XrfResult<()> {
    let mut is_available: i32 = 0;

    NgxResult::check(
      unsafe {
        match self.api {
          NgxApi::Vulkan => NVSDK_NGX_VULKAN_GetCapabilityParameters(&mut self.parameters),
          #[cfg(windows)]
          NgxApi::D3d12 => NVSDK_NGX_D3D12_GetCapabilityParameters(&mut self.parameters),
        }
      },
      "NGX has no parameters",
    )?;

    NgxResult::check(
      unsafe {
        NVSDK_NGX_Parameter_GetI(
          self.parameters,
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

    let mut is_reconstructing: i32 = 0;
    let asked: NVSDK_NGX_Result = unsafe {
      NVSDK_NGX_Parameter_GetI(
        self.parameters,
        NVSDK_NGX_Parameter_SuperSamplingDenoising_Available.as_ptr().cast(),
        &mut is_reconstructing,
      )
    };

    self.has_ray_reconstruction = asked == NVSDK_NGX_Result_NVSDK_NGX_Result_Success && is_reconstructing != 0;

    Ok(())
  }

  /// Starts NGX on a Vulkan device, through the instance and the physical device it was opened on.
  fn init_vulkan(application: &NgxApplication, device: &wgpu::Device) -> XrfResult<()> {
    unsafe {
      let hal = device
        .as_hal::<Vulkan>()
        .ok_or_else(|| XrfError::new_unexpected_error("DLSS needs a Vulkan device"))?;
      let shared = hal.shared_instance();
      let instance = shared.raw_instance();
      let engine_version = application.get_engine_version();

      application.with_feature_info(Default::default(), |info| {
        NgxResult::check(
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
      })
    }
  }

  /// Starts NGX on a D3D12 device.
  #[cfg(windows)]
  fn init_d3d12(application: &NgxApplication, device: &wgpu::Device) -> XrfResult<()> {
    unsafe {
      let hal = device
        .as_hal::<Dx12>()
        .ok_or_else(|| XrfError::new_unexpected_error("DLSS needs a D3D12 device"))?;
      let engine_version = application.get_engine_version();

      application.with_feature_info(Default::default(), |info| {
        NgxResult::check(
          NVSDK_NGX_D3D12_Init_with_ProjectID(
            application.get_project().as_ptr(),
            NVSDK_NGX_EngineType_NVSDK_NGX_ENGINE_TYPE_CUSTOM,
            engine_version.as_ptr(),
            info.ApplicationDataPath,
            hal.raw_device().as_raw().cast(),
            info.FeatureInfo,
            NVSDK_NGX_Version_NVSDK_NGX_Version_API,
          ),
          "NGX does not start",
        )
      })
    }
  }

  /// Asks the driver, on a thread of its own and once a process, to fetch newer super resolution and ray reconstruction
  /// libraries where NVIDIA offers them, holding NGX's lock as every call into it does.
  fn update(application: &NgxApplication) {
    static UPDATED: Once = Once::new();

    UPDATED.call_once(|| {
      let application: NgxApplication = application.clone();

      std::thread::spawn(move || {
        let _ngx: MutexGuard<'static, ()> = NgxLock::hold();
        let started: Instant = Instant::now();

        for feature in [
          NVSDK_NGX_Feature_NVSDK_NGX_Feature_SuperSampling,
          NVSDK_NGX_Feature_NVSDK_NGX_Feature_RayReconstruction,
        ] {
          application.with_feature_info(feature, |info| unsafe {
            NVSDK_NGX_UpdateFeature(&info.Identifier, info.FeatureID);
          });
        }

        log::debug!("DLSS asked for newer libraries in {} ms", started.elapsed().as_millis());
      });
    });
  }

  /// Takes a dropped feature to release once the device has finished its work, then releases those that are, holding
  /// NGX's lock.
  pub(crate) fn retire(&mut self, feature: NgxRetiredFeature) {
    self.retired.push(feature);
    self.release_finished();
  }

  /// Releases the retired features whose work the device has finished, holding NGX's lock.
  pub(crate) fn release_finished(&mut self) {
    let api: NgxApi = self.api;

    self.retired.retain(|feature| {
      if !feature.is_done() {
        return true;
      }

      unsafe { Self::release(api, feature.handle) };

      false
    });
  }

  /// Releases a feature made on `api`, logging what NGX refuses.
  ///
  /// # Safety
  ///
  /// NGX's lock is held, and nothing recorded with the feature is still running on the device.
  unsafe fn release(api: NgxApi, handle: *mut NVSDK_NGX_Handle) {
    let result: NVSDK_NGX_Result = unsafe {
      match api {
        NgxApi::Vulkan => NVSDK_NGX_VULKAN_ReleaseFeature(handle),
        #[cfg(windows)]
        NgxApi::D3d12 => NVSDK_NGX_D3D12_ReleaseFeature(handle),
      }
    };

    if let Err(error) = NgxResult::check(result, "DLSS does not release a feature") {
      log::error!("{error}");
    }
  }

  /// Releases the parameters, where it was given them, and shuts NGX down on the device, logging what NGX refuses.
  ///
  /// # Safety
  ///
  /// Nothing NGX recorded is still running on the device.
  unsafe fn shut_down(&mut self) {
    unsafe {
      if !self.parameters.is_null()
        && let Err(error) = NgxResult::check(
          match self.api {
            NgxApi::Vulkan => NVSDK_NGX_VULKAN_DestroyParameters(self.parameters),
            #[cfg(windows)]
            NgxApi::D3d12 => NVSDK_NGX_D3D12_DestroyParameters(self.parameters),
          },
          "NGX's parameters are not released",
        )
      {
        log::error!("{error}");
      }

      let result: Option<NVSDK_NGX_Result> = match self.api {
        NgxApi::Vulkan => self
          .device
          .as_hal::<Vulkan>()
          .map(|hal| NVSDK_NGX_VULKAN_Shutdown1(hal.raw_device().handle())),
        #[cfg(windows)]
        NgxApi::D3d12 => self
          .device
          .as_hal::<Dx12>()
          .map(|hal| NVSDK_NGX_D3D12_Shutdown1(hal.raw_device().as_raw().cast())),
      };

      if let Some(result) = result
        && let Err(error) = NgxResult::check(result, "NGX does not shut down")
      {
        log::error!("{error}");
      }
    }
  }
}

impl Drop for NgxSdk {
  /// Waits for what was submitted to finish, through wgpu, which owns the queue, then releases the retired features and
  /// shuts NGX down.
  fn drop(&mut self) {
    if let Err(error) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
      log::error!("DLSS shuts down on a device that does not finish its work: {error}");
    }

    let _ngx: MutexGuard<'static, ()> = NgxLock::hold();

    for feature in self.retired.drain(..) {
      unsafe { Self::release(self.api, feature.handle) };
    }

    unsafe { self.shut_down() };
  }
}
