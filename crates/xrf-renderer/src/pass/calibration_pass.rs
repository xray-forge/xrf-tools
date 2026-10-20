use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::frame::view_targets::ViewTargets;
use crate::pass::calibration_parameters::CalibrationParameters;
use crate::pass::calibration_uniform::CalibrationUniform;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Draws the calibration props into the G-buffer and its depth, each as a quad: a sphere ray-cast within its own,
/// writing the depth it meets, and the chart flat.
pub struct CalibrationPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  generation: u64,
}

impl CalibrationPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = CalibrationParameters::create_layout(source.device);

    Ok(Self {
      pipeline: Self::create_pipeline(source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipeline again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last one where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Calibration props rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// Draws every prop, one instance each.
  pub fn record(&self, context: &mut RasterContext<'_>, view: &ViewBinding, parameters: &CalibrationParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..6, 0..CalibrationUniform::PROPS as u32);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    source.get_render_pipeline(
      &RenderPipelineDescription::new("calibration", "frame/calibration", "vs_calibration")
        .with_fragment("fs_calibration")
        .with_bind_group_layouts(&[Some(view_layout), Some(layout)])
        .with_targets(ViewTargets::get_gbuffer_targets(true).to_vec())
        .with_depth_stencil(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(true),
          depth_compare: Some(wgpu::CompareFunction::Greater),
          stencil: Default::default(),
          bias: Default::default(),
        }),
    )
  }
}
