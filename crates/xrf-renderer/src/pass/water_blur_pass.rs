use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, PassParameters, UniformBinding,
};

use crate::frame::water_reflection::WaterReflection;
use crate::pass::enhanced_water_uniform::EnhancedWaterUniform;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::water_blur_parameters::WaterBlurParameters;
use crate::pass::water_blur_uniform::WaterBlurUniform;
use crate::shader::shader_library::ShaderLibrary;

/// Blurs the enhanced water's reflection one way at a time into half-size targets.
pub struct WaterBlurPass {
  pipeline: wgpu::RenderPipeline,
  sampler: wgpu::Sampler,
}

impl WaterBlurPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("water blur"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
    })
  }

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile, keeping the last pipeline.
  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult {
    self.pipeline = Self::create_pipeline(device, shaders)?;

    Ok(())
  }

  /// Declares one way of the blur, from `source` into a transient of `size`, and answers that transient.
  pub fn add<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    runtime: &mut GraphRuntime,
    (name, source, (width, height), way): (&'static str, GraphTexture, (u32, u32), WaterBlurUniform),
    enhanced: UniformBinding<EnhancedWaterUniform>,
  ) -> GraphTexture {
    let target: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      name,
      width,
      height,
      WaterReflection::FORMAT,
    ));
    let parameters: WaterBlurParameters<'a> = WaterBlurParameters {
      source,
      source_sampler: &self.sampler,
      enhanced,
      blur: runtime.push_uniform(&way),
    };

    graph
      .add_raster_pass(name)
      .color(GraphColorAttachment::new(
        target,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .parameters(&parameters)
      .record(move |context| {
        context.bind(&parameters);
        context.get_pass().set_pipeline(&self.pipeline);
        context.get_pass().draw(0..3, 0..1);
      });

    target
  }

  fn create_pipeline(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      device,
      shaders,
      "frame/water_blur",
      "fs_water_blur",
      &[Some(&WaterBlurParameters::create_layout(device))],
      WaterReflection::FORMAT,
    )
  }
}
