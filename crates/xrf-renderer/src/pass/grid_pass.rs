use xrf_error::{XrfError, XrfResult};

use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Draws the ground grid and a plain sky over a whole viewport.
pub struct GridPass {
  format: wgpu::TextureFormat,
  view_layout: wgpu::BindGroupLayout,
  pipeline: wgpu::RenderPipeline,
  /// The shader library generation the pipeline was built from.
  generation: u64,
}

impl GridPass {
  /// # Errors
  ///
  /// Returns an error when the shader does not compose.
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
  ) -> XrfResult<Self> {
    Ok(Self {
      pipeline: Self::create_pipeline(device, shaders, view_layout, format)?,
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      format,
    })
  }

  /// Builds the pipeline again from a library reloaded since it was built; a broken edit keeps the last good one.
  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() == self.generation {
      return;
    }

    self.generation = shaders.get_generation();

    match Self::create_pipeline(device, shaders, &self.view_layout, self.format) {
      Ok(pipeline) => self.pipeline = pipeline,
      Err(error) => log::error!("Grid shader rejected, drawing with the last one: {error}"),
    }
  }

  pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, view: &ViewBinding) {
    pass.set_pipeline(&self.pipeline);
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.draw(0..3, 0..1);
  }

  fn create_pipeline(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
  ) -> XrfResult<wgpu::RenderPipeline> {
    let source: String = shaders.compose("grid/grid", &[])?;

    let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);

    let module: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
      label: Some("grid"),
      source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("grid"),
      bind_group_layouts: &[Some(view_layout)],
      ..Default::default()
    });
    let pipeline: wgpu::RenderPipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("grid"),
      layout: Some(&layout),
      vertex: wgpu::VertexState {
        module: &module,
        entry_point: Some("vs_main"),
        compilation_options: Default::default(),
        buffers: &[],
      },
      fragment: Some(wgpu::FragmentState {
        module: &module,
        entry_point: Some("fs_main"),
        compilation_options: Default::default(),
        targets: &[Some(format.into())],
      }),
      primitive: Default::default(),
      depth_stencil: None,
      multisample: Default::default(),
      multiview_mask: None,
      cache: None,
    });

    match pollster::block_on(scope.pop()) {
      None => Ok(pipeline),
      Some(error) => Err(XrfError::new_invalid_error(format!("Grid pipeline: {error}"))),
    }
  }
}
