use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters, RasterContext};

use crate::frame::view_targets::ViewTargets;
use crate::pass::light_binning_parameters::LightBinningParameters;
use crate::pass::lights_parameters::LightsParameters;
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Clusters the view is cut into, and invocations a binning workgroup runs, as the shaders declare them.
const LIGHT_CLUSTERS: u32 = 16 * 9 * 24;
const BINNING_WORKGROUP: u32 = 64;

/// Bins the local lights in view into clusters of the view, then adds every light reaching each pixel to the light its
/// frame accumulated after the sun.
pub struct LightsPass {
  binning_layout: wgpu::BindGroupLayout,
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  binning: wgpu::ComputePipeline,
  /// Draws without contact shadows, and with them: the march's registers cost every pixel even where it never runs.
  pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl LightsPass {
  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let binning_layout: wgpu::BindGroupLayout = LightBinningParameters::create_layout(device);
    let layout: wgpu::BindGroupLayout = LightsParameters::create_layout(device);
    let (binning, pipelines) =
      Self::create_pipelines(device, shaders, &binning_layout, view_layout, &layout, texture_layout)?;

    Ok(Self {
      binning,
      pipelines,
      binning_layout,
      layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: shaders.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(
        device,
        shaders,
        &self.binning_layout,
        &self.view_layout,
        &self.layout,
        &self.texture_layout,
      ) {
        Ok((binning, pipelines)) => {
          self.binning = binning;
          self.pipelines = pipelines;
        }
        Err(error) => log::error!("Lights rejected, lighting with the last one: {error}"),
      }
    }
  }

  /// Bins the lights, then adds them over what the sun left in the light target.
  /// Bins the lights into the view's clusters.
  pub fn record_binning(&self, context: &mut ComputeContext<'_>, parameters: &LightBinningParameters) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_pipeline(&self.binning);
    pass.dispatch_workgroups(LIGHT_CLUSTERS.div_ceil(BINNING_WORKGROUP), 1, 1);
  }

  /// Adds every binned light's light to the light target the pass draws into, marching contact shadows towards the
  /// strongest where `is_contact`.
  pub fn record_draw(
    &self,
    context: &mut RasterContext<'_>,
    view: &ViewBinding,
    parameters: &LightsParameters<'_>,
    (textures, is_contact): (&wgpu::BindGroup, bool),
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

    pass.set_pipeline(&self.pipelines[is_contact as usize]);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(2, textures, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    binning_layout: &wgpu::BindGroupLayout,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(wgpu::ComputePipeline, [wgpu::RenderPipeline; 2])> {
    let binning_module: wgpu::ShaderModule = create_module(device, shaders, "frame/light_binning")?;
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/lights")?;
    let binning_pipeline_layout: wgpu::PipelineLayout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("light binning"),
        bind_group_layouts: &[Some(binning_layout)],
        ..Default::default()
      });
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("lights"),
      bind_group_layouts: &[Some(view_layout), Some(layout), Some(texture_layout)],
      ..Default::default()
    });
    let binning: wgpu::ComputePipeline = create_checked(device, "light binning", || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("light binning"),
        layout: Some(&binning_pipeline_layout),
        module: &binning_module,
        entry_point: Some("bin"),
        compilation_options: Default::default(),
        cache: None,
      })
    })?;
    // Each light adds to what the sun and the lights before it left.
    let additive: wgpu::BlendComponent = wgpu::BlendComponent {
      src_factor: wgpu::BlendFactor::One,
      dst_factor: wgpu::BlendFactor::One,
      operation: wgpu::BlendOperation::Add,
    };
    let create = |is_contact: bool| -> XrfResult<wgpu::RenderPipeline> {
      create_checked(device, "lights", || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some("lights"),
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("fs_lights"),
            compilation_options: wgpu::PipelineCompilationOptions {
              constants: &[("CONTACT_MARCHED", f64::from(u8::from(is_contact)))],
              ..Default::default()
            },
            targets: &[Some(wgpu::ColorTargetState {
              format: ViewTargets::LIGHT,
              blend: Some(wgpu::BlendState {
                color: additive,
                alpha: additive,
              }),
              write_mask: wgpu::ColorWrites::ALL,
            })],
          }),
          primitive: Default::default(),
          depth_stencil: None,
          multisample: Default::default(),
          multiview_mask: None,
          cache: None,
        })
      })
    };

    Ok((binning, [create(false)?, create(true)?]))
  }
}
