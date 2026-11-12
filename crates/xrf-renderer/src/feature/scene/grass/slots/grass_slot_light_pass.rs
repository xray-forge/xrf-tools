use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineSet, RasterContext};

use crate::feature::scene::grass::slots::grass_slot_light_parameters::GrassSlotLightParameters;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::build::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::pass::view::view_binding::ViewBinding;

/// Writes over the light target, for a debug view, the light value each drawn pixel's detail slot stores, the slot found
/// by where the pixel stands; the present shows it in place of the frame.
pub(crate) struct GrassSlotLightPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: PipelineSet<wgpu::RenderPipeline>,
  /// What the grid and the records bind while a level has no grass: a grid of no cells finds no slot.
  empty: wgpu::Buffer,
}

impl GrassSlotLightPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub(crate) fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = GrassSlotLightParameters::create_layout(source.device);

    Ok(Self {
      pipeline: PipelineSet::new(source.cache, Self::create_pipeline(source, view_layout, &layout)?),
      view_layout: view_layout.clone(),
      layout,
      empty: source.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("grass slot light empty"),
        size: 4,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
      }),
    })
  }

  /// The grid and the records bound while a level has no grass.
  pub(crate) fn get_empty(&self) -> &wgpu::Buffer {
    &self.empty
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSet::refresh`), keeping
  /// the last ones where a reload is refused.
  pub(crate) fn refresh(&mut self, source: PipelineSource<'_>) {
    if let Err(error) = self.pipeline.refresh(source.cache, || {
      Self::create_pipeline(source, &self.view_layout, &self.layout)
    }) {
      log::error!("Grass slot light rejected, drawing with the last: {error}");
    }
  }

  /// Writes every pixel of the light target the pass draws into.
  pub(crate) fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &GrassSlotLightParameters,
  ) {
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
      "grass/slot_light",
      "fs_slot_light",
      &[Some(view_layout), Some(layout)],
      ViewTargets::LIGHT,
    )
  }
}
