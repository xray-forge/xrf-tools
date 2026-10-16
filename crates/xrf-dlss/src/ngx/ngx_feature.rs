use std::ptr;
use std::sync::{Arc, Mutex};

#[cfg(windows)]
use wgpu::hal::api::Dx12;
use wgpu::hal::api::Vulkan;
#[cfg(windows)]
use windows_core::Interface;
use xrf_error::{XrfError, XrfResult};

use crate::ngx::bindings::*;
use crate::ngx::ngx_api::NgxApi;
use crate::ngx::ngx_commands::NgxCommands;
use crate::ngx::ngx_result::check_ngx;
use crate::ngx::ngx_retired_feature::NgxRetiredFeature;
use crate::ngx::ngx_sdk::NgxSdk;
use crate::ngx::ngx_sdk_guard::NgxSdkGuard;

/// A feature NGX made on a device, which it releases once the device has finished what was submitted with it.
pub(crate) struct NgxFeature {
  handle: *mut NVSDK_NGX_Handle,
  pub(crate) api: NgxApi,
  pub(crate) sdk: Arc<Mutex<NgxSdk>>,
  device: wgpu::Device,
  queue: wgpu::Queue,
}

// SAFETY: the handle is private and reaches NGX only through `evaluate` and `drop`, both holding NGX's lock, and NGX's
// parameters only under the SDK's mutex as well.
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
    let locked: NgxSdkGuard<'_> = NgxSdkGuard::lock(sdk);
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
      queue: queue.clone(),
    })
  }

  /// Records NGX's evaluation of the feature through `evaluate`, given the feature's handle, into a command buffer of
  /// its own, answered, which must be submitted right after the commands that prepared its inputs. NGX records into a
  /// fresh encoder used for nothing else, so whatever state it leaves the list in no later wgpu command meets.
  pub(crate) fn evaluate(
    &self,
    what: &str,
    evaluate: impl FnOnce(NgxCommands, &NgxSdk, *mut NVSDK_NGX_Handle) -> XrfResult<NVSDK_NGX_Result>,
  ) -> XrfResult<wgpu::CommandBuffer> {
    let mut sdk: NgxSdkGuard<'_> = NgxSdkGuard::lock(&self.sdk);

    sdk.release_finished();

    let mut commands: wgpu::CommandEncoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("dlss") });
    let result: NVSDK_NGX_Result = record(self.api, &mut commands, |list| evaluate(list, &sdk, self.handle))??;

    check_ngx(result, what)?;

    Ok(commands.finish())
  }
}

impl Drop for NgxFeature {
  /// Hands the feature to the SDK, which releases it once the device has finished what was submitted with it, rather
  /// than waiting for the device here.
  fn drop(&mut self) {
    NgxSdkGuard::lock(&self.sdk).retire(NgxRetiredFeature::new(self.handle, &self.queue));
  }
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
