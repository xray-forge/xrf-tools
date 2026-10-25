use std::ptr;
use std::sync::{Arc, Mutex};

use xrf_error::XrfResult;

use crate::dlss_quality::DlssQuality;
#[cfg(windows)]
use crate::ngx::backend::ngx_d3d12_resource::NgxD3d12Resource;
use crate::ngx::backend::ngx_vulkan_resource::NgxVulkanResource;
use crate::ngx::bindings::*;
use crate::ngx::feature::ngx_commands::NgxCommands;
use crate::ngx::feature::ngx_feature::NgxFeature;
use crate::ngx::sdk::ngx_sdk::NgxSdk;
use crate::upscaler::dlss_inputs::DlssInputs;
use crate::upscaler::dlss_preset::DlssPreset;

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
    let [optimal, min, max] = NgxFeature::query_render_sizes(
      sdk,
      (output, quality),
      "DLSS has no render size for the output",
      NGX_DLSS_GET_OPTIMAL_SETTINGS,
    )?;
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
        NgxFeature::set_preset_hints(ngx, &Self::PRESET_HINTS, preset.to_hint() as u32);

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
    NgxFeature::transition_inputs(encoder, inputs.get_read(), inputs.output);

    self
      .feature
      .evaluate("DLSS refused the frame", |commands, sdk, handle| match commands {
        NgxCommands::Vulkan(list) => Ok(Self::evaluate_vulkan(list, (handle, sdk, adapter), inputs)),
        #[cfg(windows)]
        NgxCommands::D3d12(list) => Ok(Self::evaluate_d3d12(list, (handle, sdk), inputs)),
      })
  }

  /// NGX's upscale of `inputs` recorded into a Vulkan command buffer.
  fn evaluate_vulkan(
    list: ash::vk::CommandBuffer,
    (handle, sdk, adapter): (*mut NVSDK_NGX_Handle, &NgxSdk, &wgpu::Adapter),
    inputs: &DlssInputs<'_>,
  ) -> NVSDK_NGX_Result {
    // NGX reads these through pointers while it records, so they live until it is done.
    let (mut color, mut depth, mut motion, mut output) = unsafe {
      (
        NgxVulkanResource::of(inputs.color, adapter),
        NgxVulkanResource::of(inputs.depth, adapter),
        NgxVulkanResource::of(inputs.motion, adapter),
        NgxVulkanResource::of(inputs.output, adapter),
      )
    };
    let mut bias = inputs
      .bias_current_color
      .map(|view| unsafe { NgxVulkanResource::of(view, adapter) });
    let origin = NVSDK_NGX_Coordinates { X: 0, Y: 0 };
    // SAFETY: the parameters are plain C data, whose null pointers and zeros NGX reads as the inputs left out.
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
      InColorSubrectBase: origin,
      InDepthSubrectBase: origin,
      InMVSubrectBase: origin,
      InTranslucencySubrectBase: origin,
      InBiasCurrentColorSubrectBase: origin,
      InOutputSubrectBase: origin,
      InToneMapperType: NVSDK_NGX_ToneMapperType_NVSDK_NGX_TONEMAPPER_STRING,
      pInBiasCurrentColorMask: bias.as_mut().map_or(ptr::null_mut(), ptr::from_mut),
      InFrameTimeDeltaInMsec: inputs.frame_time.as_secs_f32() * 1000.0,
      // The optional inputs left out: null resources and zero scalars.
      ..unsafe { std::mem::zeroed() }
    };

    unsafe { NGX_VULKAN_EVALUATE_DLSS_EXT(list, handle, sdk.parameters, &mut parameters) }
  }

  /// NGX's upscale of `inputs` recorded into a D3D12 command list.
  #[cfg(windows)]
  fn evaluate_d3d12(
    list: *mut ID3D12GraphicsCommandList,
    (handle, sdk): (*mut NVSDK_NGX_Handle, &NgxSdk),
    inputs: &DlssInputs<'_>,
  ) -> NVSDK_NGX_Result {
    let origin = NVSDK_NGX_Coordinates { X: 0, Y: 0 };

    unsafe {
      let mut parameters = NVSDK_NGX_D3D12_DLSS_Eval_Params {
        Feature: NVSDK_NGX_D3D12_Feature_Eval_Params {
          pInColor: NgxD3d12Resource::of(inputs.color),
          pInOutput: NgxD3d12Resource::of(inputs.output),
          InSharpness: 0.0,
        },
        pInDepth: NgxD3d12Resource::of(inputs.depth),
        pInMotionVectors: NgxD3d12Resource::of(inputs.motion),
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
          .map_or(ptr::null_mut(), |view| NgxD3d12Resource::of(view)),
        InFrameTimeDeltaInMsec: inputs.frame_time.as_secs_f32() * 1000.0,
        // The optional inputs left out: null resources and zero scalars.
        ..std::mem::zeroed()
      };

      NGX_D3D12_EVALUATE_DLSS_EXT(list, handle, sdk.parameters, &mut parameters)
    }
  }
}
