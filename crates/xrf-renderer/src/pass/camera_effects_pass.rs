use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::frame::view_targets::ViewTargets;
use crate::pass::camera_effects_parameters::CameraEffectsParameters;
use crate::pass::camera_effects_stage::CameraEffectsStage;
use crate::pass::pipeline_source::PipelineSource;

/// The camera the resolved frame is shown through: its depth of field, its motion blur and its lens, each stage a
/// triangle over its target. Its pipelines come from the pipeline cache, one a stage.
pub struct CameraEffectsPass {
  layout: wgpu::BindGroupLayout,
  /// Filtered, clamped to the edge.
  sampler: wgpu::Sampler,
}

impl CameraEffectsPass {
  /// Every target: half floats, as the scene's.
  pub const TARGET: wgpu::TextureFormat = ViewTargets::SCENE;

  const MODULE: &'static str = "frame/camera_effects";

  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      layout: CameraEffectsParameters::create_layout(device),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("camera effects"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
    }
  }

  /// Its pipelines, one a stage at the stage's index.
  pub fn describe_pipelines(&self) -> [RenderPipelineDescription; CameraEffectsStage::COUNT] {
    CameraEffectsStage::ALL.map(|stage| {
      RenderPipelineDescription::new(stage.get_pass(), Self::MODULE, "vs_fullscreen")
        .with_fragment(stage.get_fragment())
        .with_bind_group_layouts(&[Some(&self.layout)])
        .with_target(wgpu::ColorTargetState::from(Self::TARGET))
    })
  }

  /// Its pipelines as `describe_pipelines` lists them, asked of the cache.
  ///
  /// # Errors
  ///
  /// Returns the first stage's error where the module cannot be composed or wgpu refuses it.
  pub fn get_pipelines(
    &self,
    source: PipelineSource<'_>,
  ) -> XrfResult<[wgpu::RenderPipeline; CameraEffectsStage::COUNT]> {
    let pipelines: Vec<wgpu::RenderPipeline> = self
      .describe_pipelines()
      .iter()
      .map(|description| source.get_render_pipeline(description))
      .collect::<XrfResult<_>>()?;

    Ok(
      pipelines
        .try_into()
        .unwrap_or_else(|_| unreachable!("one pipeline a stage")),
    )
  }

  /// The sampler its stages read with, which their parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Draws one of its stages with the stage's `pipeline` into the target the pass draws into.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    pipeline: &wgpu::RenderPipeline,
    parameters: &CameraEffectsParameters<'_>,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.draw(0..3, 0..1);
  }
}
