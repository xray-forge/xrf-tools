use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[cfg(windows)]
use wgpu::hal::api::Dx12;
use wgpu::hal::api::Vulkan;
#[cfg(windows)]
use windows_core::Interface;
use xrf_error::{XrfError, XrfResult};

use crate::ngx::bindings::*;
use crate::ngx::ngx_api::NgxApi;
use crate::ngx::ngx_result::check_ngx;
use crate::ngx::ngx_sdk::NgxSdk;

/// A feature NGX made on a device, which it releases once the device has finished what was submitted with it.
pub(crate) struct NgxFeature {
  pub(crate) handle: *mut NVSDK_NGX_Handle,
  pub(crate) api: NgxApi,
  pub(crate) sdk: Arc<Mutex<NgxSdk>>,
  device: wgpu::Device,
}

// SAFETY: the handle is only used under the mutex its owner keeps it in, and NGX's parameters under the SDK's.
unsafe impl Send for NgxFeature {}
unsafe impl Sync for NgxFeature {}

impl NgxFeature {
  /// Makes a feature through `create`, which records its making into a command list of the device's API, on a command
  /// buffer submitted at once; `what` names it in NGX's refusal.
  pub(crate) fn create(
    sdk: &Arc<Mutex<NgxSdk>>,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    what: &str,
    create: impl FnOnce(NgxCommands, *mut NVSDK_NGX_Parameter, &mut *mut NVSDK_NGX_Handle) -> NVSDK_NGX_Result,
  ) -> XrfResult<Self> {
    let locked: MutexGuard<'_, NgxSdk> = sdk.lock().unwrap_or_else(PoisonError::into_inner);
    let api: NgxApi = locked.api;
    let mut handle: *mut NVSDK_NGX_Handle = ptr::null_mut();
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("dlss create"),
    });
    let parameters: *mut NVSDK_NGX_Parameter = locked.parameters;
    let result: NVSDK_NGX_Result = record(api, &mut encoder, |commands| create(commands, parameters, &mut handle))?;

    check_ngx(result, what)?;
    queue.submit([encoder.finish()]);
    drop(locked);

    Ok(Self {
      handle,
      api,
      sdk: Arc::clone(sdk),
      device: device.clone(),
    })
  }

  /// Records NGX's evaluation of the feature through `evaluate` into a command buffer of its own, answered, which must
  /// be submitted right after the commands that prepared its inputs.
  pub(crate) fn evaluate(
    &self,
    what: &str,
    evaluate: impl FnOnce(NgxCommands, &NgxSdk) -> XrfResult<NVSDK_NGX_Result>,
  ) -> XrfResult<wgpu::CommandBuffer> {
    let sdk: MutexGuard<'_, NgxSdk> = self.sdk.lock().unwrap_or_else(PoisonError::into_inner);
    let mut commands: wgpu::CommandEncoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("dlss") });
    let result: NVSDK_NGX_Result = record(self.api, &mut commands, |list| evaluate(list, &sdk))??;

    check_ngx(result, what)?;

    Ok(commands.finish())
  }
}

impl Drop for NgxFeature {
  /// Waits for what was submitted to finish, through wgpu, which owns the queue, then releases the feature.
  fn drop(&mut self) {
    if let Err(error) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
      log::error!("DLSS releases a feature on a device that does not finish its work: {error}");
    }

    unsafe {
      if let Err(error) = check_ngx(
        match self.api {
          NgxApi::Vulkan => NVSDK_NGX_VULKAN_ReleaseFeature(self.handle),
          #[cfg(windows)]
          NgxApi::D3d12 => NVSDK_NGX_D3D12_ReleaseFeature(self.handle),
        },
        "DLSS does not release a feature",
      ) {
        log::error!("{error}");
      }
    }
  }
}

/// The command list NGX records into, of the device's API.
#[derive(Clone, Copy)]
pub(crate) enum NgxCommands {
  Vulkan(ash::vk::CommandBuffer),
  #[cfg(windows)]
  D3d12(*mut ID3D12GraphicsCommandList),
}

/// Runs `record` with `encoder`'s command list as NGX takes it on `api`.
fn record<T>(api: NgxApi, encoder: &mut wgpu::CommandEncoder, record: impl FnOnce(NgxCommands) -> T) -> XrfResult<T> {
  unsafe {
    match api {
      NgxApi::Vulkan => encoder.as_hal_mut::<Vulkan, _, _>(|encoder| {
        let encoder = encoder.ok_or_else(|| XrfError::new_unexpected_error("DLSS records on Vulkan here"))?;

        Ok(record(NgxCommands::Vulkan(encoder.raw_handle())))
      }),
      #[cfg(windows)]
      NgxApi::D3d12 => encoder.as_hal_mut::<Dx12, _, _>(|encoder| {
        let list = encoder
          .and_then(|encoder| encoder.raw_handle())
          .ok_or_else(|| XrfError::new_unexpected_error("DLSS records on D3D12 here"))?;

        Ok(record(NgxCommands::D3d12(list.as_raw().cast())))
      }),
    }
  }
}
