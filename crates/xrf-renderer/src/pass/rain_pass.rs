use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext, RenderPipelineDescription};

use crate::frame::view_targets::ViewTargets;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::rain_parameters::RainParameters;
use crate::pass::view_binding::ViewBinding;

/// Draws the rain over a viewport's finished scene as `RenderLast` does after the forward surfaces: the streaks, then
/// the splashes, blended by their alpha and tested against the G-buffer's depth without writing it.
pub struct RainPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  /// The streaks', then the splashes'.
  pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl RainPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let layout: wgpu::BindGroupLayout = RainParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(source, view_layout, &layout)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("rain"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
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
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Rain rejected, falling with the last one: {error}"),
      }
    }
  }

  /// The sampler the streak and the splash are read through, which the parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Draws the streaks falling, and the splashes where the model has indices.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &RainParameters<'_>,
    (streaks, splash_indices): (u32, u32),
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_pipeline(&self.pipelines[0]);
    pass.draw(0..streaks * 6, 0..1);

    if splash_indices > 0 {
      pass.set_pipeline(&self.pipelines[1]);
      pass.draw(0..streaks * splash_indices, 0..1);
    }
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    // `blend(true, srcalpha, invsrcalpha)`, the alpha kept under it.
    let target: wgpu::ColorTargetState = wgpu::ColorTargetState {
      format: ViewTargets::SCENE,
      blend: Some(wgpu::BlendState {
        color: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::SrcAlpha,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
          src_factor: wgpu::BlendFactor::One,
          dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
          operation: wgpu::BlendOperation::Add,
        },
      }),
      write_mask: wgpu::ColorWrites::ALL,
    };
    // Both sides, as the engine's `CULL_NONE`.
    let create = |vertex: &'static str, fragment: &'static str| -> XrfResult<wgpu::RenderPipeline> {
      source.get_render_pipeline(
        &RenderPipelineDescription::new("rain", "frame/rain", vertex)
          .with_fragment(fragment)
          .with_bind_group_layouts(&[Some(view_layout), Some(layout)])
          .with_depth_stencil(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::Greater),
            stencil: Default::default(),
            bias: Default::default(),
          })
          .with_target(target.clone()),
      )
    };

    Ok([create("vs_streak", "fs_streak")?, create("vs_splash", "fs_splash")?])
  }
}
