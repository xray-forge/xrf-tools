use std::ffi::c_uint;
use std::ptr;
use std::sync::{Arc, Mutex};

use ash::vk;
#[cfg(windows)]
use wgpu::hal::api::Dx12;
use wgpu::hal::api::Vulkan;
#[cfg(windows)]
use windows_core::Interface;
use xrf_error::{XrfError, XrfResult};

use crate::dlss_quality::DlssQuality;
use crate::ngx::backend::ngx_api::NgxApi;
use crate::ngx::bindings::*;
use crate::ngx::feature::ngx_commands::NgxCommands;
use crate::ngx::feature::ngx_retired_feature::NgxRetiredFeature;
use crate::ngx::ngx_result::NgxResult;
use crate::ngx::sdk::ngx_sdk::NgxSdk;
use crate::ngx::sdk::ngx_sdk_guard::NgxSdkGuard;

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
    let result: NVSDK_NGX_Result =
      Self::record(api, &mut encoder, |commands| create(commands, parameters, &mut handle))?;

    NgxResult::check(result, what)?;
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

    // Taken before the encoder's hal guard: two at once are a recursive read lock.
    let vulkan: Option<ash::Device> = match self.api {
      NgxApi::Vulkan => unsafe { self.device.as_hal::<Vulkan>() }.map(|device| device.raw_device().clone()),
      #[cfg(windows)]
      NgxApi::D3d12 => None,
    };
    let mut commands: wgpu::CommandEncoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("dlss") });
    let result: NVSDK_NGX_Result = Self::record(self.api, &mut commands, |list| {
      // NGX clears and writes its output with transfer and compute commands that wgpu's barriers around it, made for
      // the states its inputs and output are declared in, do not order.
      if let (Some(device), NgxCommands::Vulkan(buffer)) = (&vulkan, list) {
        unsafe { Self::record_vulkan_barrier(device, buffer) };
      }

      let result: XrfResult<NVSDK_NGX_Result> = evaluate(list, &sdk, self.handle);

      if let (Some(device), NgxCommands::Vulkan(buffer)) = (&vulkan, list) {
        unsafe { Self::record_vulkan_barrier(device, buffer) };
      }

      result
    })??;

    NgxResult::check(result, what)?;

    Ok(commands.finish())
  }

  /// The optimal, the smallest and the largest render size a feature upscales from into `output` at `quality`, as
  /// `query` answers them for the kind of feature; the output's size alone for DLAA, which draws at it.
  pub(crate) fn query_render_sizes(
    sdk: &Mutex<NgxSdk>,
    (output, quality): ((u32, u32), DlssQuality),
    what: &str,
    query: unsafe extern "C" fn(
      *mut NVSDK_NGX_Parameter,
      c_uint,
      c_uint,
      NVSDK_NGX_PerfQuality_Value,
      *mut c_uint,
      *mut c_uint,
      *mut c_uint,
      *mut c_uint,
      *mut c_uint,
      *mut c_uint,
      *mut f32,
    ) -> NVSDK_NGX_Result,
  ) -> XrfResult<[(u32, u32); 3]> {
    let locked: NgxSdkGuard<'_> = NgxSdkGuard::lock(sdk);
    let mut optimal: (u32, u32) = (0, 0);
    let mut max: (u32, u32) = (0, 0);
    let mut min: (u32, u32) = (0, 0);
    let mut sharpness: f32 = 0.0;

    NgxResult::check(
      unsafe {
        query(
          locked.parameters,
          output.0,
          output.1,
          quality.to_value(),
          &mut optimal.0,
          &mut optimal.1,
          &mut max.0,
          &mut max.1,
          &mut min.0,
          &mut min.1,
          &mut sharpness,
        )
      },
      what,
    )?;

    // Asked for DLAA too, as NGX's own helpers record the request on its parameters, then answered as it draws.
    if quality == DlssQuality::Native {
      return Ok([output; 3]);
    }

    Ok([optimal, min, max])
  }

  /// Sets each of a feature's render preset hints, one a quality, to the preset asked, on NGX's parameters.
  ///
  /// Called while the feature is made, under the lock it is made under, so another feature made meanwhile cannot
  /// change the hints first.
  ///
  /// # Safety
  ///
  /// `parameters` are NGX's, and NGX's lock is held.
  pub(crate) unsafe fn set_preset_hints(parameters: *mut NVSDK_NGX_Parameter, hints: &[&[u8]], preset: u32) {
    for hint in hints {
      unsafe { NVSDK_NGX_Parameter_SetUI(parameters, hint.as_ptr().cast(), preset) };
    }
  }

  /// Records into `encoder` the states NGX reads `inputs` and writes `output` in, which wgpu then carries on from:
  /// sampled for the inputs, storage for the output.
  pub(crate) fn transition_inputs(
    encoder: &mut wgpu::CommandEncoder,
    inputs: Vec<&wgpu::TextureView>,
    output: &wgpu::TextureView,
  ) {
    encoder.transition_resources(
      std::iter::empty(),
      inputs
        .into_iter()
        .map(|view| wgpu::TextureTransition {
          texture: view.texture(),
          selector: None,
          state: wgpu::TextureUses::RESOURCE,
        })
        .chain([wgpu::TextureTransition {
          texture: output.texture(),
          selector: None,
          state: wgpu::TextureUses::STORAGE_READ_WRITE,
        }]),
    );
  }

  /// Orders every command recorded into `buffer` before against every command after, layouts unchanged.
  ///
  /// # Safety
  ///
  /// `buffer` is recording on `device`, outside a render pass.
  unsafe fn record_vulkan_barrier(device: &ash::Device, buffer: vk::CommandBuffer) {
    let barrier: vk::MemoryBarrier<'_> = vk::MemoryBarrier::default()
      .src_access_mask(vk::AccessFlags::MEMORY_WRITE)
      .dst_access_mask(vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE);

    unsafe {
      device.cmd_pipeline_barrier(
        buffer,
        vk::PipelineStageFlags::ALL_COMMANDS,
        vk::PipelineStageFlags::ALL_COMMANDS,
        vk::DependencyFlags::empty(),
        &[barrier],
        &[],
        &[],
      );
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
}

impl Drop for NgxFeature {
  /// Hands the feature to the SDK, which releases it once the device has finished what was submitted with it, rather
  /// than waiting for the device here.
  fn drop(&mut self) {
    NgxSdkGuard::lock(&self.sdk).retire(NgxRetiredFeature::new(self.handle, &self.queue));
  }
}
