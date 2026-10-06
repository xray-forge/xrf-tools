use xrf_error::XrfResult;
use xrf_renderer_core::{ComputeContext, PassParameters};

use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::surroundings_parameters::SurroundingsParameters;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Invocations a workgroup runs a side, as `shaders/frame/surroundings.wgsl` declares them.
const WORKGROUP: u32 = 8;

/// Lights one face of the surroundings cube from the G-buffer the static scene drew into it, the sky behind, into the
/// prefiltered cube's first mip, which the mips after it blur.
pub struct SurroundingsPass {
  layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::ComputePipeline,
  generation: u64,
}

impl SurroundingsPass {
  pub const CULL_PASS: &'static str = "surroundings cull";
  pub const DRAW_PASS: &'static str = "surroundings g-buffer";
  pub const RESOLVE_PASS: &'static str = "surroundings resolve";

  /// # Errors
  ///
  /// Returns an error when the shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let layout: wgpu::BindGroupLayout = SurroundingsParameters::create_layout(device);

    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, view_layout, &layout)?,
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipeline(device, shaders, &self.view_layout, &self.layout) {
        Ok(pipeline) => self.pipeline = pipeline,
        Err(error) => log::error!("Surroundings rejected, resolving with the last one: {error}"),
      }
    }
  }

  /// Resolves one face, `size` texels a side, through the face's camera.
  pub fn record(
    &self,
    context: &mut ComputeContext<'_>,
    view: &ViewBinding,
    size: u32,
    parameters: &SurroundingsParameters<'_>,
  ) {
    context.bind(parameters);

    let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_pipeline(&self.pipeline);
    pass.dispatch_workgroups(size.div_ceil(WORKGROUP), size.div_ceil(WORKGROUP), 1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<wgpu::ComputePipeline> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/surroundings")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("surroundings"),
      bind_group_layouts: &[Some(view_layout), Some(layout)],
      ..Default::default()
    });

    create_checked(device, "surroundings", || {
      device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("surroundings"),
        layout: Some(&pipeline_layout),
        module: &module,
        entry_point: Some("resolve"),
        compilation_options: Default::default(),
        cache: None,
      })
    })
  }
}
