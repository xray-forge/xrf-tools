use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[cfg(windows)]
use wgpu::hal::api::Dx12;
use wgpu::hal::api::Vulkan;
#[cfg(windows)]
use windows_core::Interface;
use xrf_error::{XrfError, XrfResult};

use crate::dlss_inputs::DlssInputs;
use crate::dlss_preset::DlssPreset;
use crate::dlss_quality::DlssQuality;
use crate::ngx::bindings::*;
#[cfg(windows)]
use crate::ngx::d3d12_resource::to_d3d12_resource;
use crate::ngx::ngx_api::NgxApi;
use crate::ngx::ngx_result::check_ngx;
use crate::ngx::ngx_sdk::NgxSdk;
use crate::ngx::vulkan_resource::to_ngx_resource;

/// NGX's super resolution feature for one output: its handle, the API it runs on, and the render sizes it upscales
/// from.
pub(crate) struct NgxSuperResolution {
  feature: *mut NVSDK_NGX_Handle,
  api: NgxApi,
  sdk: Arc<Mutex<NgxSdk>>,
  device: wgpu::Device,
  pub(crate) render_range: ((u32, u32), (u32, u32)),
}

// SAFETY: the handle is only used under the mutex its owner keeps it in, and NGX's parameters under the SDK's.
unsafe impl Send for NgxSuperResolution {}
unsafe impl Sync for NgxSuperResolution {}

impl NgxSuperResolution {
  /// NGX's render preset hints, one a quality, each set to the preset asked before a feature is made.
  const PRESET_HINTS: [&'static [u8]; 6] = [
    NVSDK_NGX_Parameter_DLSS_Hint_Render_Preset_DLAA,
    NVSDK_NGX_Parameter_DLSS_Hint_Render_Preset_Quality,
    NVSDK_NGX_Parameter_DLSS_Hint_Render_Preset_Balanced,
    NVSDK_NGX_Parameter_DLSS_Hint_Render_Preset_Performance,
    NVSDK_NGX_Parameter_DLSS_Hint_Render_Preset_UltraPerformance,
    NVSDK_NGX_Parameter_DLSS_Hint_Render_Preset_UltraQuality,
  ];

  /// Makes the feature for `output` pixels at a quality and preset with `flags`, on a command buffer submitted at
  /// once.
  pub(crate) fn new(
    sdk: &Arc<Mutex<NgxSdk>>,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    (output, quality, preset): ((u32, u32), DlssQuality, DlssPreset),
    flags: NVSDK_NGX_DLSS_Feature_Flags,
  ) -> XrfResult<Self> {
    let locked: MutexGuard<'_, NgxSdk> = sdk.lock().unwrap_or_else(PoisonError::into_inner);
    let api: NgxApi = locked.api;
    let value: NVSDK_NGX_PerfQuality_Value = quality.to_value();
    let mut optimal: (u32, u32) = (0, 0);
    let mut max: (u32, u32) = (0, 0);
    let mut min: (u32, u32) = (0, 0);
    let mut sharpness: f32 = 0.0;

    unsafe {
      check_ngx(
        NGX_DLSS_GET_OPTIMAL_SETTINGS(
          locked.parameters,
          output.0,
          output.1,
          value,
          &mut optimal.0,
          &mut optimal.1,
          &mut max.0,
          &mut max.1,
          &mut min.0,
          &mut min.1,
          &mut sharpness,
        ),
        "DLSS has no render size for the output",
      )?;

      for hint in Self::PRESET_HINTS {
        NVSDK_NGX_Parameter_SetUI(locked.parameters, hint.as_ptr().cast(), preset.to_hint() as u32);
      }
    }

    // DLAA draws at the output's size alone.
    if quality == DlssQuality::Native {
      (optimal, min, max) = (output, output, output);
    }

    let mut parameters = NVSDK_NGX_DLSS_Create_Params {
      Feature: NVSDK_NGX_Feature_Create_Params {
        InWidth: optimal.0,
        InHeight: optimal.1,
        InTargetWidth: output.0,
        InTargetHeight: output.1,
        InPerfQualityValue: value,
      },
      InFeatureCreateFlags: flags,
      InEnableOutputSubrects: false,
    };
    let mut feature: *mut NVSDK_NGX_Handle = ptr::null_mut();
    let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some("dlss create"),
    });
    let result: XrfResult<NVSDK_NGX_Result> = unsafe {
      match api {
        NgxApi::Vulkan => encoder.as_hal_mut::<Vulkan, _, _>(|encoder| {
          let encoder = encoder.ok_or_else(|| XrfError::new_unexpected_error("DLSS records on Vulkan here"))?;

          Ok(NGX_VULKAN_CREATE_DLSS_EXT(
            encoder.raw_handle(),
            1,
            1,
            &mut feature,
            locked.parameters,
            &mut parameters,
          ))
        }),
        #[cfg(windows)]
        NgxApi::D3d12 => encoder.as_hal_mut::<Dx12, _, _>(|encoder| {
          let list = encoder
            .and_then(|encoder| encoder.raw_handle())
            .ok_or_else(|| XrfError::new_unexpected_error("DLSS records on D3D12 here"))?;

          Ok(NGX_D3D12_CREATE_DLSS_EXT(
            list.as_raw().cast(),
            1,
            1,
            &mut feature,
            locked.parameters,
            &mut parameters,
          ))
        }),
      }
    };

    check_ngx(result?, "DLSS cannot upscale to the output")?;
    queue.submit([encoder.finish()]);
    drop(locked);

    Ok(Self {
      feature,
      api,
      sdk: Arc::clone(sdk),
      device: device.clone(),
      render_range: (min, max),
    })
  }

  /// Records the upscale of `inputs`: their barriers into `encoder`, NGX's work into the command buffer answered,
  /// which must be submitted right after `encoder`'s commands.
  pub(crate) fn record(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    adapter: &wgpu::Adapter,
    inputs: &DlssInputs<'_>,
  ) -> XrfResult<wgpu::CommandBuffer> {
    let sdk: MutexGuard<'_, NgxSdk> = self.sdk.lock().unwrap_or_else(PoisonError::into_inner);

    // NGX samples the inputs and writes the output as storage, in states wgpu then carries on from.
    encoder.transition_resources(
      std::iter::empty(),
      [inputs.color, inputs.depth, inputs.motion]
        .into_iter()
        .map(|view| wgpu::TextureTransition {
          texture: view.texture(),
          selector: None,
          state: wgpu::TextureUses::RESOURCE,
        })
        .chain([wgpu::TextureTransition {
          texture: inputs.output.texture(),
          selector: None,
          state: wgpu::TextureUses::STORAGE_READ_WRITE,
        }]),
    );

    let mut commands: wgpu::CommandEncoder = self
      .device
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("dlss") });
    let result: NVSDK_NGX_Result = match self.api {
      NgxApi::Vulkan => self.evaluate_vulkan(&mut commands, (&sdk, adapter), inputs)?,
      #[cfg(windows)]
      NgxApi::D3d12 => self.evaluate_d3d12(&mut commands, &sdk, inputs)?,
    };

    check_ngx(result, "DLSS refused the frame")?;

    Ok(commands.finish())
  }

  /// Records NGX's upscale of `inputs` into `commands`, a Vulkan encoder, answering NGX's result.
  fn evaluate_vulkan(
    &self,
    commands: &mut wgpu::CommandEncoder,
    (sdk, adapter): (&NgxSdk, &wgpu::Adapter),
    inputs: &DlssInputs<'_>,
  ) -> XrfResult<NVSDK_NGX_Result> {
    // NGX reads these through pointers while it records, so they live until it is done.
    let (mut color, mut depth, mut motion, mut output) = unsafe {
      (
        to_ngx_resource(inputs.color, adapter),
        to_ngx_resource(inputs.depth, adapter),
        to_ngx_resource(inputs.motion, adapter),
        to_ngx_resource(inputs.output, adapter),
      )
    };
    let origin = NVSDK_NGX_Coordinates { X: 0, Y: 0 };
    let mut parameters = NVSDK_NGX_VK_DLSS_Eval_Params {
      Feature: NVSDK_NGX_VK_Feature_Eval_Params {
        pInColor: &mut color,
        pInOutput: &mut output,
        InSharpness: 0.0,
      },
      pInDepth: &mut depth,
      pInMotionVectors: &mut motion,
      InJitterOffsetX: inputs.jitter[0],
      InJitterOffsetY: inputs.jitter[1],
      InRenderSubrectDimensions: NVSDK_NGX_Dimensions {
        Width: inputs.render.0,
        Height: inputs.render.1,
      },
      InReset: i32::from(inputs.is_reset),
      InMVScaleX: inputs.motion_scale[0],
      InMVScaleY: inputs.motion_scale[1],
      pInTransparencyMask: ptr::null_mut(),
      pInExposureTexture: ptr::null_mut(),
      pInBiasCurrentColorMask: ptr::null_mut(),
      InColorSubrectBase: origin,
      InDepthSubrectBase: origin,
      InMVSubrectBase: origin,
      InTranslucencySubrectBase: origin,
      InBiasCurrentColorSubrectBase: origin,
      InOutputSubrectBase: origin,
      InPreExposure: 0.0,
      InExposureScale: 0.0,
      InIndicatorInvertXAxis: 0,
      InIndicatorInvertYAxis: 0,
      GBufferSurface: NVSDK_NGX_VK_GBuffer {
        pInAttrib: [ptr::null_mut(); 17],
      },
      InToneMapperType: NVSDK_NGX_ToneMapperType_NVSDK_NGX_TONEMAPPER_STRING,
      pInMotionVectors3D: ptr::null_mut(),
      pInIsParticleMask: ptr::null_mut(),
      pInAnimatedTextureMask: ptr::null_mut(),
      pInDepthHighRes: ptr::null_mut(),
      pInPositionViewSpace: ptr::null_mut(),
      InFrameTimeDeltaInMsec: 0.0,
      pInRayTracingHitDistance: ptr::null_mut(),
      pInMotionVectorsReflections: ptr::null_mut(),
    };

    unsafe {
      commands.as_hal_mut::<Vulkan, _, _>(|commands| {
        let commands = commands.ok_or_else(|| XrfError::new_unexpected_error("DLSS records on Vulkan here"))?;

        Ok(NGX_VULKAN_EVALUATE_DLSS_EXT(
          commands.raw_handle(),
          self.feature,
          sdk.parameters,
          &mut parameters,
        ))
      })
    }
  }

  /// Records NGX's upscale of `inputs` into `commands`, a D3D12 encoder, answering NGX's result.
  #[cfg(windows)]
  fn evaluate_d3d12(
    &self,
    commands: &mut wgpu::CommandEncoder,
    sdk: &NgxSdk,
    inputs: &DlssInputs<'_>,
  ) -> XrfResult<NVSDK_NGX_Result> {
    let origin = NVSDK_NGX_Coordinates { X: 0, Y: 0 };
    let mut parameters = unsafe {
      NVSDK_NGX_D3D12_DLSS_Eval_Params {
        Feature: NVSDK_NGX_D3D12_Feature_Eval_Params {
          pInColor: to_d3d12_resource(inputs.color),
          pInOutput: to_d3d12_resource(inputs.output),
          InSharpness: 0.0,
        },
        pInDepth: to_d3d12_resource(inputs.depth),
        pInMotionVectors: to_d3d12_resource(inputs.motion),
        InJitterOffsetX: inputs.jitter[0],
        InJitterOffsetY: inputs.jitter[1],
        InRenderSubrectDimensions: NVSDK_NGX_Dimensions {
          Width: inputs.render.0,
          Height: inputs.render.1,
        },
        InReset: i32::from(inputs.is_reset),
        InMVScaleX: inputs.motion_scale[0],
        InMVScaleY: inputs.motion_scale[1],
        InColorSubrectBase: origin,
        InDepthSubrectBase: origin,
        InMVSubrectBase: origin,
        InTranslucencySubrectBase: origin,
        InBiasCurrentColorSubrectBase: origin,
        InOutputSubrectBase: origin,
        InToneMapperType: NVSDK_NGX_ToneMapperType_NVSDK_NGX_TONEMAPPER_STRING,
        // The optional inputs left out: null resources and zero scalars.
        ..std::mem::zeroed()
      }
    };

    unsafe {
      commands.as_hal_mut::<Dx12, _, _>(|commands| {
        let list = commands
          .and_then(|commands| commands.raw_handle())
          .ok_or_else(|| XrfError::new_unexpected_error("DLSS records on D3D12 here"))?;

        Ok(NGX_D3D12_EVALUATE_DLSS_EXT(
          list.as_raw().cast(),
          self.feature,
          sdk.parameters,
          &mut parameters,
        ))
      })
    }
  }
}

impl Drop for NgxSuperResolution {
  /// Waits for what was submitted to finish, through wgpu, which owns the queue, then releases the feature.
  fn drop(&mut self) {
    if let Err(error) = self.device.poll(wgpu::PollType::wait_indefinitely()) {
      log::error!("DLSS releases an upscaler on a device that does not finish its work: {error}");
    }

    unsafe {
      if let Err(error) = check_ngx(
        match self.api {
          NgxApi::Vulkan => NVSDK_NGX_VULKAN_ReleaseFeature(self.feature),
          #[cfg(windows)]
          NgxApi::D3d12 => NVSDK_NGX_D3D12_ReleaseFeature(self.feature),
        },
        "DLSS does not release an upscaler",
      ) {
        log::error!("{error}");
      }
    }
  }
}
