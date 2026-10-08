use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sun_shafts_parameters::SunShaftsParameters;
use crate::pass::view_binding::ViewBinding;

/// Adds the sun's light shafts to a viewport's frame after its forward surfaces, as `phase_combine_volumetric` adds
/// what `accum_direct_volumetric` gathered: `one, one`; marched at half the frame's size first.
pub struct SunShaftsPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  /// The half-size march, then the composite adding it to the frame.
  pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl SunShaftsPass {
  /// What the half-size march gathers: how much of the shafts' light each texel's ray takes.
  pub const MARCHED: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SunShaftsParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(device, source, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(device, source, &self.view_layout, &self.layout) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Sun shafts rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// Draws the half-size march where `is_march`, the composite adding it to the frame otherwise.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &SunShaftsParameters,
    is_march: bool,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[usize::from(!is_march)]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let added: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };

    let target = |format: wgpu::TextureFormat| {
      Some(wgpu::ColorTargetState {
        format,
        blend: Some(wgpu::BlendState {
          color: added,
          alpha: added,
        }),
        write_mask: wgpu::ColorWrites::COLOR,
      })
    };

    Ok([
      create_fullscreen_pipeline_into(
        device,
        source,
        "frame/sun_shafts",
        "fs_sun_shafts_march",
        &[Some(view_layout), Some(layout)],
        &[Some(Self::MARCHED.into())],
      )?,
      create_fullscreen_pipeline_into(
        device,
        source,
        "frame/sun_shafts",
        "fs_sun_shafts",
        &[Some(view_layout), Some(layout)],
        &[target(ViewTargets::SCENE), target(ViewTargets::HIGH)],
      )?,
    ])
  }
}
