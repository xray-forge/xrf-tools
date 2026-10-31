use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::pass::fullscreen_pipeline::create_fullscreen_pipeline_into;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::sky_behind_parameters::SkyBehindParameters;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::view_binding::ViewBinding;

/// Draws the sky the distance fog fades into along each pixel's view, at half the frame's size, for combine to mix in.
pub struct SkyBehindPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sky_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  /// What combine reads while no sky shows behind the fog: never read.
  unseen: wgpu::TextureView,
  generation: u64,
}

impl SkyBehindPass {
  /// The sky's colour, at half floats.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    sky_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = SkyBehindParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(source, view_layout, &layout, sky_layout)?,
      view_layout: view_layout.clone(),
      sky_layout: sky_layout.clone(),
      unseen: device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some("sky behind unseen"),
          size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: Self::FORMAT,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default()),
      generation: source.get_generation(),
      layout,
    })
  }

  /// Makes its pipelines again where the shader library was reloaded since (`PipelineSource::get_generation`), keeping
  /// the last ones where a reload is refused.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipeline(source, &self.view_layout, &self.layout, &self.sky_layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Sky behind rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// What combine reads while no sky shows behind the fog.
  pub fn get_unseen(&self) -> &wgpu::TextureView {
    &self.unseen
  }

  /// Draws what far geometry fades into behind the fog: the haze above the fold, the sky as drawn under it.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &SkyBehindParameters,
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
      "frame/sky_behind",
      "fs_sky_behind",
      &[Some(view_layout), Some(layout), Some(sky_layout)],
      &[Some(Self::FORMAT.into())],
    )
  }
}
