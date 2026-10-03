use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::flare_uniform::FLARE_SLOTS;
use crate::pass::fullscreen_pipeline::{buffer_binding, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::level_shadows::LevelShadows;
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
    let compute: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE;
    let stages: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let measure_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("flare visibility"),
      entries: &[
        uniform_entry(0, compute),
        storage_entry(1, compute, true),
        texture_entry(
          2,
          compute,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2,
        ),
        texture_entry(
          3,
          compute,
          wgpu::TextureSampleType::Depth,
          wgpu::TextureViewDimension::D2Array,
        ),
        uniform_entry(4, compute),
      ],
    });
    let draw_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("flare"),
      entries: &[
        uniform_entry(0, stages),
        storage_entry(1, wgpu::ShaderStages::VERTEX, false),
        wgpu::BindGroupLayoutEntry {
          binding: 2,
          visibility: wgpu::ShaderStages::FRAGMENT,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
      ],
    });
    let texture_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("flare texture"),
      entries: &[texture_entry(
        0,
        wgpu::ShaderStages::FRAGMENT,
        wgpu::TextureSampleType::Float { filterable: true },
        wgpu::TextureViewDimension::D2,
      )],
    });
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

  /// What the measure reads and writes: the flare's uniform, the eased visibility, the frame's depth and the sun's
  /// shadow.
  pub fn create_measure_group(
    &self,
    device: &wgpu::Device,
    (uniform, state): (&wgpu::Buffer, &wgpu::Buffer),
    targets: &ViewTargets,
    shadows: &LevelShadows,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("flare visibility"),
      layout: &self.measure_layout,
      entries: &[
        buffer_binding(0, uniform),
        buffer_binding(1, state),
        texture_binding(2, &targets.depth),
        texture_binding(3, &shadows.get_maps().view),
        buffer_binding(4, shadows.get_uniform()),
      ],
    })
  }

  /// What every flare's draw reads: the flare's uniform, the eased visibility and the sampler.
  pub fn create_draw_group(
    &self,
    device: &wgpu::Device,
    uniform: &wgpu::Buffer,
    state: &wgpu::Buffer,
  ) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("flare"),
      layout: &self.draw_layout,
      entries: &[
        buffer_binding(0, uniform),
        buffer_binding(1, state),
        wgpu::BindGroupEntry {
          binding: 2,
          resource: wgpu::BindingResource::Sampler(&self.sampler),
        },
      ],
    })
  }

  /// One flare's texture.
  pub fn create_texture_group(&self, device: &wgpu::Device, texture: &wgpu::TextureView) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("flare texture"),
      layout: &self.texture_layout,
      entries: &[texture_binding(0, texture)],
    })
  }

  /// Measures how much of the sun shows this frame, eased from the last.
  pub fn measure(&self, encoder: &mut wgpu::CommandEncoder, view: &ViewBinding, group: &wgpu::BindGroup) {
    let mut pass: wgpu::ComputePass<'_> = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
      label: Some("flare visibility"),
      timestamp_writes: None,
    });

    pass.set_pipeline(&self.measure);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, group, &[]);
    pass.dispatch_workgroups(1, 1, 1);
  }

  /// Draws each flare, by instance, with its texture's group; the gradient is `GRADIENT_INSTANCE`.
  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    group: &wgpu::BindGroup,
    draws: &[(u32, &wgpu::BindGroup)],
  ) {
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("flares"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &targets.scene,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });

    pass.set_pipeline(&self.draw);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, group, &[]);

    for (instance, texture) in draws {
      pass.set_bind_group(2, *texture, &[]);
      pass.draw(0..6, *instance..*instance + 1);
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
