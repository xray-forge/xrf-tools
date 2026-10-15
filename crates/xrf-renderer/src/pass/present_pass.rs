use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::present_parameters::PresentParameters;
use crate::pass::view_binding::ViewBinding;

/// Puts a viewport's finished scene into its rectangle of the window, moved where the water and the particles distort
/// it and dithered to the window's eight bits; or, for a debug view, one of the targets the scene was built from.
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
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> Self {
    let device: &wgpu::Device = source.device;

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
      generation: source.get_generation(),
    }
  }

  /// Builds the pipelines again from a library reloaded since they were built, one a window format each; a broken edit
  /// keeps the last good ones.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() == self.generation {
      return;
    }

    self.generation = source.get_generation();

    let rebuilt: XrfResult<HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>> = self
      .pipelines
      .keys()
      .map(|format| Ok((*format, self.create_pipeline(source, *format)?)))
      .collect();

    match rebuilt {
      Ok(pipelines) => self.pipelines = pipelines,
      Err(error) => log::error!("Present shader rejected, drawing with the last one: {error}"),
    }
  }

  /// Builds the pipeline drawing into a window format, unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn prepare(&mut self, source: PipelineSource<'_>, format: wgpu::TextureFormat) -> XrfResult {
    if !self.pipelines.contains_key(&format) {
      let pipeline: wgpu::RenderPipeline = self.create_pipeline(source, format)?;

      self.pipelines.insert(format, pipeline);
    }

    Ok(())
  }

  fn create_pipeline(
    &self,
    source: PipelineSource<'_>,
    format: wgpu::TextureFormat,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline(
      source,
      "frame/present",
      "fs_present",
      &[Some(&self.view_layout), Some(&self.layout)],
      format,
    )
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
