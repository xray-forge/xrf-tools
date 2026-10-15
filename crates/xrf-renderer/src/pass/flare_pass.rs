use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, ComputePipelineDescription, PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::flare_draw_parameters::FlareDrawParameters;
use crate::pass::flare_measure_parameters::FlareMeasureParameters;
use crate::pass::flare_texture_parameters::FlareTextureParameters;
use crate::pass::flare_uniform::FlareUniform;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::shader_pipelines::create_checked;
use crate::pass::view_binding::ViewBinding;

/// Measures how much of the sun shows and draws the lens flares and the gradient over a viewport's finished frame, as
/// `CEnvironment::RenderFlares` does after combine: each a quad added by its alpha, `srcalpha, one`, with no depth.
pub struct FlarePass {
  measure_layout: wgpu::BindGroupLayout,
  draw_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  sampler: wgpu::Sampler,
  measure: wgpu::ComputePipeline,
  draw: wgpu::RenderPipeline,
  generation: u64,
}

impl FlarePass {
  /// The instance the gradient is drawn as, as `shaders/frame/flare.wgsl` reads it.
  pub const GRADIENT_INSTANCE: u32 = FlareUniform::SLOTS as u32;

  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(source: PipelineSource<'_>, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let device: &wgpu::Device = source.device;

    let measure_layout: wgpu::BindGroupLayout = FlareMeasureParameters::create_layout(device);
    let draw_layout: wgpu::BindGroupLayout = FlareDrawParameters::create_layout(device);
    let texture_layout: wgpu::BindGroupLayout = FlareTextureParameters::create_layout(device);
    let (measure, draw) =
      Self::create_pipelines(source, view_layout, [&measure_layout, &draw_layout, &texture_layout])?;

    Ok(Self {
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("flare"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      generation: source.get_generation(),
      measure_layout,
      draw_layout,
      texture_layout,
      measure,
      draw,
    })
  }

  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    if source.get_generation() != self.generation {
      self.generation = source.get_generation();

      match Self::create_pipelines(
        source,
        &self.view_layout,
        [&self.measure_layout, &self.draw_layout, &self.texture_layout],
      ) {
        Ok((measure, draw)) => (self.measure, self.draw) = (measure, draw),
        Err(error) => log::error!("Lens flares rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// The sampler the flares' textures are read through, which their parameters bind.
  pub fn get_sampler(&self) -> &wgpu::Sampler {
    &self.sampler
  }

  /// Measures how much of the sun shows this frame, eased from the last.
  pub fn record_measure(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    parameters: &FlareMeasureParameters,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.measure);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
  }

  /// Draws each flare, by instance, with its texture's group; the gradient is `Self::GRADIENT_INSTANCE`.
  pub fn record_draw(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &FlareDrawParameters<'_>,
    draws: &[(u32, FlareTextureParameters)],
  ) {
    context.bind(parameters);
    context.get_pass().set_pipeline(&self.draw);
    context.get_pass().set_bind_group(0, &view.bind_group, &[]);

    for (instance, texture) in draws {
      context.bind(texture);
      context.get_pass().draw(0..6, *instance..*instance + 1);
    }
  }

  fn create_pipelines(
    source: PipelineSource<'_>,
    view_layout: &wgpu::BindGroupLayout,
    [measure_layout, draw_layout, texture_layout]: [&wgpu::BindGroupLayout; 3],
  ) -> XrfResult<(wgpu::ComputePipeline, wgpu::RenderPipeline)> {
    let device: &wgpu::Device = source.device;

    let draw_module: wgpu::ShaderModule = source.get_module("frame/flare")?;
    let draw_pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("flare"),
      bind_group_layouts: &[Some(view_layout), Some(draw_layout), Some(texture_layout)],
      ..Default::default()
    });
    let added: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::SrcAlpha,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };
    let targets: [Option<wgpu::ColorTargetState>; 1] = [Some(wgpu::ColorTargetState {
      format: ViewTargets::SCENE,
      blend: Some(wgpu::BlendState {
        color: added,
        alpha: added,
      }),
      write_mask: wgpu::ColorWrites::COLOR,
    })];
    let measure: wgpu::ComputePipeline = source.get_compute_pipeline(
      &ComputePipelineDescription::new("flare visibility", "frame/flare_visibility", "cs_visibility")
        .with_bind_group_layouts(&[Some(view_layout), Some(measure_layout)]),
    )?;
    let draw: wgpu::RenderPipeline = create_checked(device, "flare", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("flare"),
        layout: Some(&draw_pipeline_layout),
        vertex: wgpu::VertexState {
          module: &draw_module,
          entry_point: Some("vs_flare"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &draw_module,
          entry_point: Some("fs_flare"),
          compilation_options: Default::default(),
          targets: &targets,
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: source.get_driver_cache(),
      })
    })?;

    Ok((measure, draw))
  }
}
