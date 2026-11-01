use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::lighting::combine::combine_parameters::CombineParameters;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Lights a viewport's G-buffer with the light it accumulated and the hemisphere, and tonemaps it into its scene.
pub struct CombinePass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::RenderPipeline>,
}

impl CombinePass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = CombineParameters::create_layout(device);

    Ok(Self {
      pipeline: PipelineSet::new(
        source.cache,
        Self::create_pipeline(source, view_layout, &layout, sky_layout)?,
      ),
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipeline.refresh(source.cache, || {
      Self::create_pipeline(source, &self.view_layout, &self.layout, &self.sky_layout)
    }) {
      log::error!("Combine rejected, combining with the last one: {error}");
    }
  }

  /// Lights the G-buffer as `hmodel` does into the scene and its high part, the sky drawn where nothing was.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &CombineParameters<'_>,
    sky: &SkyParameters<'_>,
  ) {
    context.bind(parameters);
    context.bind(sky);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::RenderPipeline> {
    create_fullscreen_pipeline_into(
      source,
      "frame/combine",
      "fs_combine",
      &[Some(view_layout), Some(layout), Some(sky_layout)],
      &[Some(ViewTargets::SCENE.into()), Some(ViewTargets::HIGH.into())],
    )
  }
}
