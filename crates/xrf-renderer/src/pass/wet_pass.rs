use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wet_apply_parameters::WetApplyParameters;
use crate::pass::wet_patch_parameters::WetPatchParameters;
use crate::shader::shader_library::ShaderLibrary;

/// Rain on the G-buffer before any light, as `draw_rain` wets it (`r3_rendertarget_draw_rain.cpp`): where the rain
/// reaches near the camera its normals patched into the light target, borrowed before the sun clears it, then written
/// back, then the albedo darkened and the gloss raised by how wet it is. Enhanced, the same three stages wet the
/// surfaces out to the distance and gather puddles on terrain.
pub struct WetPass {
  view_layout: wgpu::BindGroupLayout,
  /// The patch's, then the write back's and the wetting's.
  layouts: [wgpu::BindGroupLayout; 2],
  sampler: wgpu::Sampler,
  /// The patch, the normal written back, the albedo and gloss wetted; the engine's, then the enhanced.
  pipelines: [[wgpu::RenderPipeline; 3]; 2],
  generation: u64,
}

impl WetPass {
  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let patch: wgpu::BindGroupLayout = WetPatchParameters::create_layout(device);
    let apply: wgpu::BindGroupLayout = WetApplyParameters::create_layout(device);
    let layouts: [wgpu::BindGroupLayout; 2] = [patch, apply];

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, view_layout, &layouts)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("wet"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layouts,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Wet surfaces rejected, wetting with the last ones: {error}"),
      }
    }
  }

  /// Draws one of its three stages into the target the pass draws into: the wet patches into the light, then the wet
  /// look into the normals and the albedo, each from the group it reads.
  /// The sampler the splash volume and the flow are read through, which the patch's parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Draws one stage, the engine's or the enhanced: the patches, bound by `P` as [`WetPatchParameters`], or a wet look,
  /// as [`WetApplyParameters`].
  pub fn record<P: PassParameters>(
    &self,
    context: &mut RasterContext<'_>,
    (stage, is_enhanced): (usize, bool),
    view: &ViewBinding,
    parameters: &P,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[usize::from(is_enhanced)][stage]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layouts: &[wgpu::BindGroupLayout; 2],
  ) -> XrfResult<[[wgpu::RenderPipeline; 3]; 2]> {
    Ok([
      Self::create_stages(device, shaders, view_layout, layouts, "")?,
      Self::create_stages(device, shaders, view_layout, layouts, "_enhanced")?,
    ])
  }

  /// The three stages whose entry points end in `suffix`.
  fn create_stages(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    [patch, apply]: &[wgpu::BindGroupLayout; 2],
    suffix: &str,
  ) -> XrfResult<[wgpu::RenderPipeline; 3]> {
    let patched: wgpu::RenderPipeline = create_fullscreen_pipeline(
      device,
      shaders,
      "frame/wet_patch",
      &format!("fs_wet_patch{suffix}"),
      &[Some(view_layout), Some(patch)],
      ViewTargets::LIGHT,
    )?;
    let normal: wgpu::RenderPipeline = create_fullscreen_pipeline(
      device,
      shaders,
      "frame/wet_apply",
      &format!("fs_wet_normal{suffix}"),
      &[Some(view_layout), Some(apply)],
      ViewTargets::NORMAL,
    )?;
    // `blend(zero, srccolor)` on colour and `(one, one)` on alpha: the albedo multiplied, the gloss added.
    let gloss: wgpu::RenderPipeline = create_fullscreen_pipeline(
      device,
      shaders,
      "frame/wet_apply",
      &format!("fs_wet_gloss{suffix}"),
      &[Some(view_layout), Some(apply)],
      wgpu::ColorTargetState {
        format: ViewTargets::ALBEDO,
        blend: Some(wgpu::BlendState {
          color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Zero,
            dst_factor: wgpu::BlendFactor::Src,
            operation: wgpu::BlendOperation::Add,
          },
          alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
          },
        }),
        write_mask: wgpu::ColorWrites::ALL,
      },
    )?;

    Ok([patched, normal, gloss])
  }
}
