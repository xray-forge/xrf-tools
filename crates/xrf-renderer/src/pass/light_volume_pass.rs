use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::light_volume_parameters::LightVolumeParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Draws the shadowed local lights' shafts (`accum_volumetric`) into a target of their own at half the frame's size,
/// slices added where the scene stands behind them, tested against its depth taken at that size without writing it,
/// for the shafts' composite to bring up and add to the frame with the sun's.
pub struct LightVolumePass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  /// The slices, then the half-size depth they are tested against.
  pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl LightVolumePass {
  /// What the shafts add up in, as `rt_Generic_2` holds them.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = LightVolumeParameters::create_layout(device);

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, [view_layout, &layout, texture_layout])?,
      layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: shaders.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, [&self.view_layout, &self.layout, &self.texture_layout]) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Light volumes rejected, drawing with the last ones: {error}"),
      }
    }
  }

  /// Draws each volume's slices, `slices` a volume in the parameters' order.
  pub fn record(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &LightVolumeParameters,
    textures: &wgpu::BindGroup,
    slices: &[u32],
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[0]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);

    for (volume, count) in slices.iter().enumerate() {
      let volume: u32 = volume as u32;

      pass.draw(0..count * 6, volume..volume + 1);
    }
  }

  /// Takes the frame's depth at half its size into the depth target the slices are tested against.
  pub fn record_depth(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &LightVolumeParameters,
    textures: &wgpu::BindGroup,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[1]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    layouts: [&wgpu::BindGroupLayout; 3],
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/light_volumes")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("light volumes"),
      bind_group_layouts: &layouts.map(Some),
      ..Default::default()
    });
    let added: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };

    let slices: wgpu::RenderPipeline = create_checked(device, "fs_light_volume", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("light volumes"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &module,
          entry_point: Some("vs_light_volume"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &module,
          entry_point: Some("fs_light_volume"),
          compilation_options: Default::default(),
          targets: &[Some(wgpu::ColorTargetState {
            format: Self::FORMAT,
            blend: Some(wgpu::BlendState {
              color: added,
              alpha: added,
            }),
            write_mask: wgpu::ColorWrites::COLOR,
          })],
        }),
        // `accum_volumetric.s`: both sides, the scene's depth tested and never written.
        primitive: wgpu::PrimitiveState {
          cull_mode: None,
          ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(false),
          depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
          stencil: Default::default(),
          bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
      })
    })?;
    let depth: wgpu::RenderPipeline = create_checked(device, "fs_volume_depth", || {
      device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("light volumes depth"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
          module: &module,
          entry_point: Some("vs_fullscreen"),
          compilation_options: Default::default(),
          buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
          module: &module,
          entry_point: Some("fs_volume_depth"),
          compilation_options: Default::default(),
          targets: &[],
        }),
        primitive: Default::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
          format: ViewTargets::DEPTH,
          depth_write_enabled: Some(true),
          depth_compare: Some(wgpu::CompareFunction::Always),
          stencil: Default::default(),
          bias: Default::default(),
        }),
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
      })
    })?;

    Ok([slices, depth])
  }
}
