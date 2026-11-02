use std::ptr;
use std::sync::{Arc, Mutex};

use wgpu::hal::api::Vulkan;
use xrf_error::XrfResult;

use crate::dlss_quality::DlssQuality;
#[cfg(windows)]
use crate::ngx::backend::ngx_d3d12_resource::NgxD3d12Resource;
use crate::ngx::backend::ngx_vulkan_resource::NgxVulkanResource;
use crate::ngx::bindings::*;
use crate::ngx::feature::ngx_commands::NgxCommands;
use crate::ngx::feature::ngx_feature::NgxFeature;
use crate::ngx::sdk::ngx_sdk::NgxSdk;
use crate::reconstruction::dlss_ray_inputs::DlssRayInputs;
use crate::reconstruction::dlss_ray_preset::DlssRayPreset;

/// NGX's ray reconstruction feature for one output: denoising and upscaling in one network, over the guides a frame
/// gives it, and the render sizes it upscales from.
pub(crate) struct NgxRayReconstruction {
  feature: NgxFeature,
  pub(crate) render_range: ((u32, u32), (u32, u32)),
}

impl NgxRayReconstruction {
  /// NGX's ray reconstruction render preset hints, one a quality, each set to the preset asked before a feature is
  /// made.
  const PRESET_HINTS: [&'static [u8]; 6] = [
    NVSDK_NGX_Parameter_RayReconstruction_Hint_Render_Preset_DLAA,
    NVSDK_NGX_Parameter_RayReconstruction_Hint_Render_Preset_Quality,
    NVSDK_NGX_Parameter_RayReconstruction_Hint_Render_Preset_Balanced,
    NVSDK_NGX_Parameter_RayReconstruction_Hint_Render_Preset_Performance,
    NVSDK_NGX_Parameter_RayReconstruction_Hint_Render_Preset_UltraPerformance,
    NVSDK_NGX_Parameter_RayReconstruction_Hint_Render_Preset_UltraQuality,
  ];

  /// Makes the feature for `output` pixels at a quality and preset, over hardware depth and roughness packed into the
  /// normals' fourth channel, with `flags`.
  pub(crate) fn new(
    sdk: &Arc<Mutex<NgxSdk>>,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    (output, quality, preset): ((u32, u32), DlssQuality, DlssRayPreset),
    flags: NVSDK_NGX_DLSS_Feature_Flags,
  ) -> XrfResult<Self> {
    let value: NVSDK_NGX_PerfQuality_Value = quality.to_value();
    // Ray reconstruction's own sizes on Windows, super resolution's where its D3D helpers are not compiled.
    #[cfg(windows)]
    let query = NGX_DLSSD_GET_OPTIMAL_SETTINGS;
    #[cfg(not(windows))]
    let query = NGX_DLSS_GET_OPTIMAL_SETTINGS;
    let [optimal, min, max] = NgxFeature::query_render_sizes(
      sdk,
      (output, quality),
      "DLSS has no render size for the output to reconstruct rays from",
      query,
    )?;
    // The Vulkan device NGX was started on, which it makes the feature on; none on D3D12, whose command list names it.
    // SAFETY: the raw device is only read for its handle, which outlives the call it is passed to.
    let vulkan_device: ash::vk::Device =
      unsafe { device.as_hal::<Vulkan>() }.map_or_else(ash::vk::Device::null, |hal| hal.raw_device().handle());
    let mut parameters = NVSDK_NGX_DLSSD_Create_Params {
      InDenoiseMode: NVSDK_NGX_DLSS_Denoise_Mode_NVSDK_NGX_DLSS_Denoise_Mode_DLUnified,
      InRoughnessMode: NVSDK_NGX_DLSS_Roughness_Mode_NVSDK_NGX_DLSS_Roughness_Mode_Packed,
      InUseHWDepth: NVSDK_NGX_DLSS_Depth_Type_NVSDK_NGX_DLSS_Depth_Type_HW,
      InWidth: optimal.0,
      InHeight: optimal.1,
      InTargetWidth: output.0,
      InTargetHeight: output.1,
      InPerfQualityValue: value,
      InFeatureCreateFlags: flags,
      InEnableOutputSubrects: false,
    };
    let feature: NgxFeature = NgxFeature::create(
      sdk,
      (device, queue),
      "DLSS cannot reconstruct rays into the output",
      |commands, ngx, handle| unsafe {
        NgxFeature::set_preset_hints(ngx, &Self::PRESET_HINTS, preset.to_hint() as u32);

        match commands {
          NgxCommands::Vulkan(list) => {
            NGX_VULKAN_CREATE_DLSSD_EXT1(vulkan_device, list, 1, 1, handle, ngx, &mut parameters)
          }
          #[cfg(windows)]
          NgxCommands::D3d12(list) => NGX_D3D12_CREATE_DLSSD_EXT(list, 1, 1, handle, ngx, &mut parameters),
        }
      },
    )?;

    Ok(Self {
      feature,
      render_range: (min, max),
    })
  }

  /// Records the reconstruction of `inputs`: their barriers into `encoder`, NGX's work into the command buffer
  /// answered, which must be submitted right after `encoder`'s commands.
  pub(crate) fn record(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    adapter: &wgpu::Adapter,
    inputs: &DlssRayInputs<'_>,
  ) -> XrfResult<wgpu::CommandBuffer> {
    NgxFeature::transition_inputs(encoder, inputs.get_read(), inputs.output);

    // NGX reads the matrices through pointers while it records, row after row.
    let mut world_to_view: [f32; 16] = inputs.world_to_view.transpose().to_cols_array();
    let mut view_to_clip: [f32; 16] = inputs.view_to_clip.transpose().to_cols_array();
    let matrices: (*mut f32, *mut f32) = (world_to_view.as_mut_ptr(), view_to_clip.as_mut_ptr());

    self.feature.evaluate(
      "DLSS refused to reconstruct the frame",
      |commands, sdk, handle| match commands {
        NgxCommands::Vulkan(list) => Ok(Self::evaluate_vulkan(list, (handle, sdk, adapter), inputs, matrices)),
        #[cfg(windows)]
        NgxCommands::D3d12(list) => Ok(Self::evaluate_d3d12(list, (handle, sdk), inputs, matrices)),
      },
    )
  }

  /// NGX's reconstruction of `inputs` recorded into a Vulkan command buffer.
  fn evaluate_vulkan(
    list: ash::vk::CommandBuffer,
    (handle, sdk, adapter): (*mut NVSDK_NGX_Handle, &NgxSdk, &wgpu::Adapter),
    inputs: &DlssRayInputs<'_>,
    (world_to_view, view_to_clip): (*mut f32, *mut f32),
  ) -> NVSDK_NGX_Result {
    // NGX reads these through pointers while it records, so they live until it is done.
    let resource = |view: &wgpu::TextureView| unsafe { NgxVulkanResource::of(view, adapter) };
    let mut color = resource(inputs.color);
    let mut output = resource(inputs.output);
    let mut depth = resource(inputs.depth);
    let mut motion = resource(inputs.motion);
    let mut diffuse_albedo = resource(inputs.diffuse_albedo);
    let mut specular_albedo = resource(inputs.specular_albedo);
    let mut normals = resource(inputs.normals);
    let mut hit_distance = resource(inputs.specular_hit_distance);
    let mut diffuse_hit_distance = inputs.diffuse_hit_distance.map(resource);
    let mut bias = inputs.bias_current_color.map(resource);
    let mut transparency = inputs
      .transparency
      .map(|(before, after)| (resource(before), resource(after)));
    let mut parameters: NVSDK_NGX_VK_DLSSD_Eval_Params = unsafe { std::mem::zeroed() };

    parameters.pInColor = &mut color;
    parameters.pInOutput = &mut output;
    parameters.pInDepth = &mut depth;
    parameters.pInMotionVectors = &mut motion;
    parameters.pInDiffuseAlbedo = &mut diffuse_albedo;
    parameters.pInSpecularAlbedo = &mut specular_albedo;
    parameters.pInNormals = &mut normals;
    parameters.pInSpecularHitDistance = &mut hit_distance;

    if let Some(diffuse) = &mut diffuse_hit_distance {
      parameters.pInDiffuseHitDistance = diffuse;
    }

    if let Some(bias) = &mut bias {
      parameters.pInBiasCurrentColorMask = bias;
    }

    if let Some((before, after)) = &mut transparency {
      parameters.pInColorBeforeTransparency = before;
      parameters.pInColorAfterTransparency = after;
    }

    parameters.pInWorldToViewMatrix = world_to_view;
    parameters.pInViewToClipMatrix = view_to_clip;
    parameters.InJitterOffsetX = inputs.jitter[0];
    parameters.InJitterOffsetY = inputs.jitter[1];
    parameters.InRenderSubrectDimensions = NVSDK_NGX_Dimensions {
      Width: inputs.render.0,
      Height: inputs.render.1,
    };
    parameters.InReset = i32::from(inputs.is_reset);
    parameters.InMVScaleX = inputs.motion_scale[0];
    parameters.InMVScaleY = inputs.motion_scale[1];
    parameters.InToneMapperType = NVSDK_NGX_ToneMapperType_NVSDK_NGX_TONEMAPPER_STRING;
    parameters.InPreExposure = inputs.pre_exposure;
    parameters.pInRoughness = ptr::null_mut();

    unsafe { NGX_VULKAN_EVALUATE_DLSSD_EXT(list, handle, sdk.parameters, &mut parameters) }
  }

  /// NGX's reconstruction of `inputs` recorded into a D3D12 command list.
  #[cfg(windows)]
  fn evaluate_d3d12(
    list: *mut ID3D12GraphicsCommandList,
    (handle, sdk): (*mut NVSDK_NGX_Handle, &NgxSdk),
    inputs: &DlssRayInputs<'_>,
    (world_to_view, view_to_clip): (*mut f32, *mut f32),
  ) -> NVSDK_NGX_Result {
    unsafe {
      // The optional inputs left out: null resources and zero scalars.
      let mut parameters: NVSDK_NGX_D3D12_DLSSD_Eval_Params = std::mem::zeroed();

      parameters.pInColor = NgxD3d12Resource::of(inputs.color);
      parameters.pInOutput = NgxD3d12Resource::of(inputs.output);
      parameters.pInDepth = NgxD3d12Resource::of(inputs.depth);
      parameters.pInMotionVectors = NgxD3d12Resource::of(inputs.motion);
      parameters.pInDiffuseAlbedo = NgxD3d12Resource::of(inputs.diffuse_albedo);
      parameters.pInSpecularAlbedo = NgxD3d12Resource::of(inputs.specular_albedo);
      parameters.pInNormals = NgxD3d12Resource::of(inputs.normals);
      parameters.pInSpecularHitDistance = NgxD3d12Resource::of(inputs.specular_hit_distance);

      if let Some(diffuse) = inputs.diffuse_hit_distance {
        parameters.pInDiffuseHitDistance = NgxD3d12Resource::of(diffuse);
      }

      if let Some(bias) = inputs.bias_current_color {
        parameters.pInBiasCurrentColorMask = NgxD3d12Resource::of(bias);
      }

      if let Some((before, after)) = inputs.transparency {
        parameters.pInColorBeforeTransparency = NgxD3d12Resource::of(before);
        parameters.pInColorAfterTransparency = NgxD3d12Resource::of(after);
      }

      parameters.pInWorldToViewMatrix = world_to_view;
      parameters.pInViewToClipMatrix = view_to_clip;
      parameters.InJitterOffsetX = inputs.jitter[0];
      parameters.InJitterOffsetY = inputs.jitter[1];
      parameters.InRenderSubrectDimensions = NVSDK_NGX_Dimensions {
        Width: inputs.render.0,
        Height: inputs.render.1,
      };
      parameters.InReset = i32::from(inputs.is_reset);
      parameters.InMVScaleX = inputs.motion_scale[0];
      parameters.InMVScaleY = inputs.motion_scale[1];
      parameters.InToneMapperType = NVSDK_NGX_ToneMapperType_NVSDK_NGX_TONEMAPPER_STRING;
      parameters.InPreExposure = inputs.pre_exposure;

      NGX_D3D12_EVALUATE_DLSSD_EXT(list, handle, sdk.parameters, &mut parameters)
    }
  }
}
