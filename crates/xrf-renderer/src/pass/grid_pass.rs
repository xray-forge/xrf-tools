use xrf_error::XrfResult;
use xrf_renderer_core::RenderPipelineDescription;

use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Draws the ground grid and a plain sky over a whole viewport.
pub struct GridPass {
  format: wgpu::TextureFormat,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  /// The shader library generation the pipeline was built from.
  generation: u64,
}

impl GridPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
  ) -> XrfResult<Self> {
    Ok(Self {
      pipeline: Self::create_pipeline(source, view_layout, format)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      format,
    })
  }

  /// Builds the pipeline again from a library reloaded since it was built; a broken edit keeps the last good one.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() == self.generation {
      return;
    }

    self.generation = source.get_generation();

    match Self::create_pipeline(source, &self.view_layout, self.format) {
      Ok(pipeline) => self.pipeline = pipeline,
      Err(error) => log::error!("Grid shader rejected, drawing with the last one: {error}"),
    }
  }

  /// Draws the ground grid over a viewport that shows no scene, into the window's image.
  pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, view: &ViewBinding) {
    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
  ) -> XrfResult<wgpu::RenderPipeline> {
    source.get_render_pipeline(
      &RenderPipelineDescription::new("grid", "grid/grid", "vs_grid")
        .with_fragment("fs_grid")
        .with_bind_group_layouts(&[Some(view_layout)])
        .with_target(format.into()),
    )
  }
}
