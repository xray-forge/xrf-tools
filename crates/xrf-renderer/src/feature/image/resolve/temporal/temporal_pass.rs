use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::image::resolve::temporal::temporal_parameters::TemporalParameters;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::pass::view::view_binding::ViewBinding;

/// Resolves a viewport's jittered frame with its history into the next history, which is the frame shown.
pub struct TemporalPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  pipeline: PipelineSet<wgpu::RenderPipeline>,
}

impl TemporalPass {
  /// Its history's format: colour and the distance along the view, at half floats.
  pub const HISTORY: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// How much each pixel flickers, which it keeps beside its history.
  pub const FLICKER: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = TemporalParameters::create_layout(device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, view_layout, &layout)?),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("temporal history"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipeline.refresh(source.cache, || {
      Self::create_pipeline(source, &self.view_layout, &self.layout)
    }) {
      log::error!("Temporal resolve rejected, resolving with the last one: {error}");
    }
  }

  /// The sampler the history is read through, which the parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Resolves this frame against its history into the history written, keeping each pixel's flicker beside it.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &TemporalParameters<'_>) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline_into(
      source,
      "frame/temporal",
      "fs_temporal",
      &[Some(view_layout), Some(layout)],
      &[Some(Self::HISTORY.into()), Some(Self::FLICKER.into())],
    )
  }
}
