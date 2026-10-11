use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::dlss_reactive_parameters::DlssReactiveParameters;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Works out the mask DLSS super resolution upscales from this frame rather than the history by: where what drew over
/// the surfaces changed the frame, as the temporal resolve's reactive weight reads it.
pub struct DlssReactivePass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl DlssReactivePass {
  /// How far a pixel is upscaled from this frame alone.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = DlssReactiveParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("DLSS reactive mask rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// Writes the bias-current-colour mask DLSS reads: how much the transparent layers changed each pixel against the frame
  /// before them.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &DlssReactiveParameters) {
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
    create_fullscreen_pipeline(
      source,
      "frame/dlss_reactive",
      "fs_dlss_reactive",
      &[Some(view_layout), Some(layout)],
      Self::FORMAT,
    )
  }
}
