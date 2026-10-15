use std::ptr;
use std::sync::{Arc, Mutex};

use xrf_error::XrfResult;

use crate::dlss_inputs::DlssInputs;
use crate::dlss_preset::DlssPreset;
use crate::dlss_quality::DlssQuality;
use crate::ngx::bindings::*;
#[cfg(windows)]
use crate::ngx::d3d12_resource::to_d3d12_resource;
use crate::ngx::ngx_feature::{NgxCommands, NgxFeature};
use crate::ngx::ngx_result::check_ngx;
use crate::ngx::ngx_sdk::NgxSdk;
use crate::ngx::ngx_sdk_guard::NgxSdkGuard;
use crate::ngx::vulkan_resource::to_ngx_resource;

/// NGX's super resolution feature for one output, and the render sizes it upscales from.
pub(crate) struct NgxSuperResolution {
  feature: NgxFeature,
  pub(crate) render_range: ((u32, u32), (u32, u32)),
}

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
    let value: NVSDK_NGX_PerfQuality_Value = quality.to_value();
    let mut optimal: (u32, u32) = (0, 0);
    let mut max: (u32, u32) = (0, 0);
    let mut min: (u32, u32) = (0, 0);
    let mut sharpness: f32 = 0.0;

    {
      let locked: NgxSdkGuard<'_> = NgxSdkGuard::lock(sdk);

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
    let feature: NgxFeature = NgxFeature::create(
      sdk,
      (device, queue),
      "DLSS cannot upscale to the output",
      |commands, ngx, handle| unsafe {
        match commands {
          NgxCommands::Vulkan(list) => NGX_VULKAN_CREATE_DLSS_EXT(list, 1, 1, handle, ngx, &mut parameters),
          #[cfg(windows)]
          NgxCommands::D3d12(list) => NGX_D3D12_CREATE_DLSS_EXT(list, 1, 1, handle, ngx, &mut parameters),
        }
      },
    )?;

    Ok(Self {
      feature,
      render_range: (min, max),
    })
  }

  /// Records the upscale of `inputs`: their barriers into `encoder`, NGX's work into the command buffer answered,
  /// which must be submitted right after `encoder`'s commands.
  pub(crate) fn record(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    adapter: &wgpu::Adapter,
    inputs: &DlssInputs<'_>,
  ) -> XrfResult<wgpu::CommandBuffer> {
    // NGX samples the inputs and writes the output as storage, in states wgpu then carries on from.
    encoder.transition_resources(
      std::iter::empty(),
      [inputs.color, inputs.depth, inputs.motion]
        .into_iter()
        .chain(inputs.bias_current_color)
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

    self
      .feature
      .evaluate("DLSS refused the frame", |commands, sdk| match commands {
        NgxCommands::Vulkan(list) => Ok(Self::evaluate_vulkan(list, (&self.feature, sdk, adapter), inputs)),
        #[cfg(windows)]
        NgxCommands::D3d12(list) => Ok(Self::evaluate_d3d12(list, (&self.feature, sdk), inputs)),
      })
  }

  /// NGX's upscale of `inputs` recorded into a Vulkan command buffer.
  fn evaluate_vulkan(
    list: ash::vk::CommandBuffer,
    (feature, sdk, adapter): (&NgxFeature, &NgxSdk, &wgpu::Adapter),
    inputs: &DlssInputs<'_>,
  ) -> NVSDK_NGX_Result {
    // NGX reads these through pointers while it records, so they live until it is done.
    let (mut color, mut depth, mut motion, mut output) = unsafe {
      (
        to_ngx_resource(inputs.color, adapter),
        to_ngx_resource(inputs.depth, adapter),
        to_ngx_resource(inputs.motion, adapter),
        to_ngx_resource(inputs.output, adapter),
      )
    };
    let mut bias = inputs
      .bias_current_color
      .map(|view| unsafe { to_ngx_resource(view, adapter) });
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
      pInBiasCurrentColorMask: bias.as_mut().map_or(ptr::null_mut(), ptr::from_mut),
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
      InFrameTimeDeltaInMsec: inputs.frame_time,
      pInRayTracingHitDistance: ptr::null_mut(),
      pInMotionVectorsReflections: ptr::null_mut(),
    };

    unsafe { NGX_VULKAN_EVALUATE_DLSS_EXT(list, feature.handle, sdk.parameters, &mut parameters) }
  }

  /// NGX's upscale of `inputs` recorded into a D3D12 command list.
  #[cfg(windows)]
  fn evaluate_d3d12(
    list: *mut ID3D12GraphicsCommandList,
    (feature, sdk): (&NgxFeature, &NgxSdk),
    inputs: &DlssInputs<'_>,
  ) -> NVSDK_NGX_Result {
    let origin = NVSDK_NGX_Coordinates { X: 0, Y: 0 };

    unsafe {
      let mut parameters = NVSDK_NGX_D3D12_DLSS_Eval_Params {
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
        pInBiasCurrentColorMask: inputs
          .bias_current_color
          .map_or(ptr::null_mut(), |view| to_d3d12_resource(view)),
        InFrameTimeDeltaInMsec: inputs.frame_time,
        // The optional inputs left out: null resources and zero scalars.
        ..std::mem::zeroed()
      };

      NGX_D3D12_EVALUATE_DLSS_EXT(list, feature.handle, sdk.parameters, &mut parameters)
    }
  }
}
