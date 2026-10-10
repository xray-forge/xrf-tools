use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_error::XrfResult;

use crate::dlss_quality::DlssQuality;
use crate::dlss_ray_inputs::DlssRayInputs;
use crate::ngx::bindings::*;
#[cfg(windows)]
use crate::ngx::d3d12_resource::to_d3d12_resource;
use crate::ngx::ngx_feature::{NgxCommands, NgxFeature};
use crate::ngx::ngx_result::check_ngx;
use crate::ngx::ngx_sdk::NgxSdk;
use crate::ngx::vulkan_resource::to_ngx_resource;

/// NGX's ray reconstruction feature for one output: denoising and upscaling in one network, over the guides a frame
/// gives it, and the render sizes it upscales from.
pub(crate) struct NgxRayReconstruction {
  feature: NgxFeature,
  pub(crate) render_range: ((u32, u32), (u32, u32)),
}

impl NgxRayReconstruction {
  /// Makes the feature for `output` pixels at a quality, over hardware depth and roughness packed into the normals'
  /// fourth channel, with `flags`.
  pub(crate) fn new(
    sdk: &Arc<Mutex<NgxSdk>>,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    (output, quality): ((u32, u32), DlssQuality),
    flags: NVSDK_NGX_DLSS_Feature_Flags,
  ) -> XrfResult<Self> {
    let value: NVSDK_NGX_PerfQuality_Value = quality.to_value();
    let [mut optimal, mut min, mut max] = Self::get_render_sizes(sdk, output, value)?;

    // DLAA draws at the output's size alone.
    if quality == DlssQuality::Native {
      (optimal, min, max) = (output, output, output);
    }

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
        match commands {
          NgxCommands::Vulkan(list) => {
            NGX_VULKAN_CREATE_DLSSD_EXT1(ash::vk::Device::null(), list, 1, 1, handle, ngx, &mut parameters)
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

  /// The optimal, the smallest and the largest render size ray reconstruction upscales from into `output` at a
  /// quality: its own on Windows, super resolution's where its D3D helpers are not compiled.
  fn get_render_sizes(
    sdk: &Arc<Mutex<NgxSdk>>,
    output: (u32, u32),
    value: NVSDK_NGX_PerfQuality_Value,
  ) -> XrfResult<[(u32, u32); 3]> {
    let locked: MutexGuard<'_, NgxSdk> = sdk.lock().unwrap_or_else(PoisonError::into_inner);
    let mut optimal: (u32, u32) = (0, 0);
    let mut max: (u32, u32) = (0, 0);
    let mut min: (u32, u32) = (0, 0);
    let mut sharpness: f32 = 0.0;

    #[cfg(windows)]
    let get = NGX_DLSSD_GET_OPTIMAL_SETTINGS;
    #[cfg(not(windows))]
    let get = NGX_DLSS_GET_OPTIMAL_SETTINGS;

    check_ngx(
      unsafe {
        get(
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
        )
      },
      "DLSS has no render size for the output to reconstruct rays from",
    )?;

    Ok([optimal, min, max])
  }

  /// Records the reconstruction of `inputs`: their barriers into `encoder`, NGX's work into the command buffer
  /// answered, which must be submitted right after `encoder`'s commands.
  pub(crate) fn record(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    adapter: &wgpu::Adapter,
    inputs: &DlssRayInputs<'_>,
  ) -> XrfResult<wgpu::CommandBuffer> {
    // NGX samples the inputs and writes the output as storage, in states wgpu then carries on from.
    encoder.transition_resources(
      std::iter::empty(),
      inputs
        .get_read()
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

    // NGX reads the matrices through pointers while it records, row after row.
    let mut world_to_view: [f32; 16] = inputs.world_to_view.transpose().to_cols_array();
    let mut view_to_clip: [f32; 16] = inputs.view_to_clip.transpose().to_cols_array();
    let matrices: (*mut f32, *mut f32) = (world_to_view.as_mut_ptr(), view_to_clip.as_mut_ptr());

    self.feature.evaluate(
      "DLSS refused to reconstruct the frame",
      |commands, sdk| match commands {
        NgxCommands::Vulkan(list) => Ok(Self::evaluate_vulkan(
          list,
          (&self.feature, sdk, adapter),
          inputs,
          matrices,
        )),
        #[cfg(windows)]
        NgxCommands::D3d12(list) => Ok(Self::evaluate_d3d12(list, (&self.feature, sdk), inputs, matrices)),
      },
    )
  }

  /// NGX's reconstruction of `inputs` recorded into a Vulkan command buffer.
  fn evaluate_vulkan(
    list: ash::vk::CommandBuffer,
    (feature, sdk, adapter): (&NgxFeature, &NgxSdk, &wgpu::Adapter),
    inputs: &DlssRayInputs<'_>,
    (world_to_view, view_to_clip): (*mut f32, *mut f32),
  ) -> NVSDK_NGX_Result {
    // NGX reads these through pointers while it records, so they live until it is done.
    let resource = |view: &wgpu::TextureView| unsafe { to_ngx_resource(view, adapter) };
    let mut color = resource(inputs.color);
    let mut output = resource(inputs.output);
    let mut depth = resource(inputs.depth);
    let mut motion = resource(inputs.motion);
    let mut diffuse_albedo = resource(inputs.diffuse_albedo);
    let mut specular_albedo = resource(inputs.specular_albedo);
    let mut normals = resource(inputs.normals);
    let mut hit_distance = resource(inputs.specular_hit_distance);
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
    parameters.pInRoughness = ptr::null_mut();

    unsafe { NGX_VULKAN_EVALUATE_DLSSD_EXT(list, feature.handle, sdk.parameters, &mut parameters) }
  }

  /// NGX's reconstruction of `inputs` recorded into a D3D12 command list.
  #[cfg(windows)]
  fn evaluate_d3d12(
    list: *mut ID3D12GraphicsCommandList,
    (feature, sdk): (&NgxFeature, &NgxSdk),
    inputs: &DlssRayInputs<'_>,
    (world_to_view, view_to_clip): (*mut f32, *mut f32),
  ) -> NVSDK_NGX_Result {
    unsafe {
      // The optional inputs left out: null resources and zero scalars.
      let mut parameters: NVSDK_NGX_D3D12_DLSSD_Eval_Params = std::mem::zeroed();

      parameters.pInColor = to_d3d12_resource(inputs.color);
      parameters.pInOutput = to_d3d12_resource(inputs.output);
      parameters.pInDepth = to_d3d12_resource(inputs.depth);
      parameters.pInMotionVectors = to_d3d12_resource(inputs.motion);
      parameters.pInDiffuseAlbedo = to_d3d12_resource(inputs.diffuse_albedo);
      parameters.pInSpecularAlbedo = to_d3d12_resource(inputs.specular_albedo);
      parameters.pInNormals = to_d3d12_resource(inputs.normals);
      parameters.pInSpecularHitDistance = to_d3d12_resource(inputs.specular_hit_distance);

      if let Some(bias) = inputs.bias_current_color {
        parameters.pInBiasCurrentColorMask = to_d3d12_resource(bias);
      }

      if let Some((before, after)) = inputs.transparency {
        parameters.pInColorBeforeTransparency = to_d3d12_resource(before);
        parameters.pInColorAfterTransparency = to_d3d12_resource(after);
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

      NGX_D3D12_EVALUATE_DLSSD_EXT(list, feature.handle, sdk.parameters, &mut parameters)
    }
  }
}
