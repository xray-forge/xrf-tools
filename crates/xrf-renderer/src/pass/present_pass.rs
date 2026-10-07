use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::present_parameters::PresentParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Puts a viewport's finished scene into its rectangle of the window, moved where the water and the particles distort it and dithered to
/// the window's eight bits; or, for a debug view, one of the targets the scene was built from.
pub struct PresentPass {
  layout: wgpu::BindGroupLayout,
  /// The bloom's, `smp_rtlinear`: filtered, clamped to the edge.
  bloom_sampler: wgpu::Sampler,
  view_layout: wgpu::BindGroupLayout,
  /// One a window format, built on first use.
  pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
  generation: u64,
}

impl PresentPass {
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> Self {
    let layout: wgpu::BindGroupLayout = PresentParameters::create_layout(device);

    Self {
      layout,
      bloom_sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("present bloom"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      pipelines: HashMap::new(),
      generation: shaders.get_generation(),
    }
  }

  /// Forgets pipelines built from a library reloaded since, so the next frame builds them again.
  pub fn refresh(&mut self, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();
      self.pipelines.clear();
    }
  }

  /// Builds the pipeline drawing into a window format, unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary, format: wgpu::TextureFormat) -> XrfResult {
    if !self.pipelines.contains_key(&format) {
      let pipeline: wgpu::RenderPipeline = create_fullscreen_pipeline(
        device,
        shaders,
        "frame/present",
        "fs_present",
        &[Some(&self.view_layout), Some(&self.layout)],
        format,
      )?;

      self.pipelines.insert(format, pipeline);
    }

    Ok(())
  }

  /// The bloom's sampler, which a viewport's parameters bind.
  pub fn get_bloom_sampler(&self) -> &wgpu::Sampler {
    &self.bloom_sampler
  }

  /// Draws into the window's pass, whose viewport and scissor are the viewport's rectangle already.
  pub fn draw(
    &self,
    context: &mut RasterContext<'_>,
    format: wgpu::TextureFormat,
    view: &ViewBinding,
    parameters: &PresentParameters<'_>,
  ) {
    if let Some(pipeline) = self.pipelines.get(&format) {
      context.get_pass().set_pipeline(pipeline);
      context.bind(parameters);

      let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

      pass.set_bind_group(0, &view.bind_group, &[]);
      pass.draw(0..3, 0..1);
    }
  }
}
