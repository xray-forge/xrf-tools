use std::collections::HashMap;

use xrf_error::XrfResult;

use crate::context::gpu_context::GpuContext;
use crate::pass::grid_pass::GridPass;
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;
use crate::window::render_window::RenderWindow;

/// Everything made on the GPU, which goes as a whole when the GPU stops or is lost.
pub struct GpuState {
  pub context: GpuContext,
  pub view_layout: wgpu::BindGroupLayout,
  /// Each window's swapchain, by its host's key.
  pub windows: HashMap<u64, RenderWindow>,
  /// The grid pass for each target format drawn into.
  grids: HashMap<wgpu::TextureFormat, GridPass>,
}

impl GpuState {
  pub fn new(context: GpuContext) -> Self {
    Self {
      view_layout: ViewBinding::create_layout(&context.device),
      windows: HashMap::new(),
      grids: HashMap::new(),
      context,
    }
  }

  /// Builds the grid pass drawing into a format, unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when its shader does not compose or compile.
  pub fn ensure_grid(&mut self, shaders: &ShaderLibrary, format: wgpu::TextureFormat) -> XrfResult {
    if !self.grids.contains_key(&format) {
      let grid: GridPass = GridPass::new(&self.context.device, shaders, &self.view_layout, format)?;

      self.grids.insert(format, grid);
    }

    Ok(())
  }

  /// The grid pass drawing into a format, once [`GpuState::ensure_grid`] built it.
  pub fn get_grid(&self, format: wgpu::TextureFormat) -> Option<&GridPass> {
    self.grids.get(&format)
  }

  /// Rebuilds pipelines from a shader library reloaded since they were built.
  pub fn refresh(&mut self, shaders: &ShaderLibrary) {
    for grid in self.grids.values_mut() {
      grid.refresh(&self.context.device, shaders);
    }
  }
}
