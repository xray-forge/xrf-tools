use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::flare_draw_parameters::FlareDrawParameters;
use crate::pass::flare_measure_parameters::FlareMeasureParameters;
use crate::pass::flare_texture_parameters::FlareTextureParameters;
use crate::pass::flare_uniform::FLARE_SLOTS;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// The instance the gradient is drawn as, as `shaders/frame/flare.wgsl` reads it.
pub const GRADIENT_INSTANCE: u32 = FLARE_SLOTS as u32;

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
  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let measure_layout: wgpu::BindGroupLayout = FlareMeasureParameters::create_layout(device);
    let draw_layout: wgpu::BindGroupLayout = FlareDrawParameters::create_layout(device);
    let texture_layout: wgpu::BindGroupLayout = FlareTextureParameters::create_layout(device);
    let (measure, draw) = Self::create_pipelines(
      device,
      shaders,
      view_layout,
      [&measure_layout, &draw_layout, &texture_layout],
    )?;

    Ok(Self {
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("flare"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      measure_layout,
      draw_layout,
      texture_layout,
      measure,
      draw,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(
        device,
        shaders,
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

  /// Draws each flare, by instance, with its texture's group; the gradient is `GRADIENT_INSTANCE`.
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
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    [measure_layout, draw_layout, texture_layout]: [&wgpu::BindGroupLayout; 3],
  ) -> XrfResult<(wgpu::ComputePipeline, wgpu::RenderPipeline)> {
    let measure_module: wgpu::ShaderModule = create_module(device, shaders, "frame/flare_visibility")?;
    let draw_module: wgpu::ShaderModule = create_module(device, shaders, "frame/flare")?;
    let measure_pipeline_layout: wgpu::PipelineLayout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("flare visibility"),
        bind_group_layouts: &[Some(view_layout), Some(measure_layout)],
        ..Default::default()
      });
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
    let measure: wgpu::ComputePipeline = create_checked(device, "flare visibility", || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("flare visibility"),
        layout: Some(&measure_pipeline_layout),
        module: &measure_module,
        entry_point: Some("cs_visibility"),
        compilation_options: Default::default(),
        cache: None,
      })
    })?;
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
        cache: None,
      })
    })?;

    Ok((measure, draw))
  }
}
