use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::ambient_occlusion_parameters::AmbientOcclusionParameters;
use crate::pass::ambient_occlusion_stage::AmbientOcclusionStage;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::view_binding::ViewBinding;

/// Ambient occlusion at half the frame's size: searched into its first target, denoised across into the second and
/// down back into the first, which combine reads.
pub struct AmbientOcclusionPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// The search, its slices and steps the uniform's.
  search: wgpu::RenderPipeline,
  /// The denoise across, then down.
  denoises: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl AmbientOcclusionPass {
  /// The module its pipelines are made from.
  const MODULE: &'static str = "frame/ambient_occlusion";

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = AmbientOcclusionParameters::create_layout(device);
    let (search, denoises) = Self::create_pipelines(source, view_layout, &layout)?;

    Ok(Self {
      search,
      denoises,
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

      match Self::create_pipelines(source, &self.view_layout, &self.layout) {
        Ok((search, denoises)) => {
          self.search = search;
          self.denoises = denoises;
        }
        Err(error) => log::error!("Ambient occlusion rejected, occluding with the last one: {error}"),
      }
    }
  }

  /// Draws one of its stages into the target the pass draws into: the search, or a denoise.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    stage: AmbientOcclusionStage,
    view: &ViewBinding,
    parameters: &AmbientOcclusionParameters,
  ) {
    let pipeline: &wgpu::RenderPipeline = match stage {
      AmbientOcclusionStage::Search => &self.search,
      AmbientOcclusionStage::DenoiseAcross => &self.denoises[0],
      AmbientOcclusionStage::DenoiseBack => &self.denoises[1],
    };

    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  #[allow(clippy::type_complexity)]
  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(wgpu::RenderPipeline, [wgpu::RenderPipeline; 2])> {
    let layouts: [Option<&wgpu::BindGroupLayout>; 2] = [Some(view_layout), Some(layout)];
    let create = |fragment: &'static str| -> XrfResult<wgpu::RenderPipeline> {
      create_fullscreen_pipeline(source, Self::MODULE, fragment, &layouts, ViewTargets::OCCLUSION)
    };

    Ok((create("fs_search")?, [create("fs_denoise_x")?, create("fs_denoise_y")?]))
  }
}
