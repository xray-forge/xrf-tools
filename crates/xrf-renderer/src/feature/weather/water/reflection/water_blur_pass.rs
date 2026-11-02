use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, PassParameters, PipelineSet,
  UniformBinding,
};

use crate::feature::weather::water::enhanced::enhanced_water_uniform::EnhancedWaterUniform;
use crate::feature::weather::water::reflection::water_blur_parameters::WaterBlurParameters;
use crate::feature::weather::water::reflection::water_blur_way::WaterBlurWay;
use crate::feature::weather::water::reflection::water_reflection::WaterReflection;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::build::pipeline_source::PipelineSource;

/// Blurs the enhanced water's reflection one way at a time into half-size targets. Its owner, the water pass, refreshes
/// it.
pub struct WaterBlurPass {
  /// Its parameters' layout, made once, so a pipeline made again keys the cache as the last did.
  layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::RenderPipeline>,
  sampler: wgpu::Sampler,
}

impl WaterBlurPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;
    let layout: wgpu::BindGroupLayout = WaterBlurParameters::create_layout(device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, &layout)?),
      layout,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("water blur"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
    })
  }

  /// Makes its pipeline again where the shader library was reloaded since (`PipelineSet::refresh`).
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile, keeping the last pipeline.
  pub fn refresh(&mut self, source: PipelineSource<'_>) -> XrfResult {
    self
      .pipeline
      .refresh(source.cache, || Self::create_pipeline(source, &self.layout))
  }

  /// Declares one way of the blur, from its source into a transient of its size, and answers that transient.
  pub fn add<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    runtime: &mut GraphRuntime,
    WaterBlurWay {
      name,
      source,
      size: (width, height),
      way,
    }: WaterBlurWay,
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

  fn create_pipeline(source: PipelineSource<'_>, layout: &wgpu::BindGroupLayout) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      source,
      "frame/water_blur",
      "fs_water_blur",
      &[Some(layout)],
      WaterReflection::FORMAT,
    )
  }
}
