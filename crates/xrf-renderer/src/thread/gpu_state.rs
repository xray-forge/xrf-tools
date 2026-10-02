use std::collections::HashMap;

use xrf_error::XrfResult;

use crate::context::gpu_context::GpuContext;
use crate::pass::combine_pass::CombinePass;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::grid_pass::GridPass;
use crate::pass::level_passes::LevelPasses;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::texture::texture_cache::TextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::window::render_window::RenderWindow;

/// Everything made on the GPU, which goes as a whole when the GPU stops or is lost.
pub struct GpuState {
  pub context: GpuContext,
  pub view_layout: wgpu::BindGroupLayout,
  /// Each window's swapchain, by its host's key.
  pub windows: HashMap<u64, RenderWindow>,
  /// Every texture any viewport's scene samples, uploaded once however many draw it.
  pub textures: TextureCache,
  pub static_cull: StaticCullPass,
  pub static_gbuffer: StaticGBufferPass,
  pub pyramid: DepthPyramidPass,
  pub combine: CombinePass,
  /// The grid pass for each target format drawn into.
  grids: HashMap<wgpu::TextureFormat, GridPass>,
}

impl GpuState {
  /// # Errors
  ///
  /// Returns an error when a pass's shaders do not compose or compile.
  pub fn new(context: GpuContext, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let device: &wgpu::Device = &context.device;
    let view_layout: wgpu::BindGroupLayout = ViewBinding::create_layout(device);
    let textures: TextureCache = TextureCache::new(device, &context.queue);

    Ok(Self {
      static_cull: StaticCullPass::new(device, shaders, &view_layout)?,
      static_gbuffer: StaticGBufferPass::new(device, shaders, &view_layout, textures.get_layout())?,
      pyramid: DepthPyramidPass::new(device, shaders)?,
      combine: CombinePass::new(device, &context.queue, shaders, &view_layout),
      windows: HashMap::new(),
      grids: HashMap::new(),
      textures,
      view_layout,
      context,
    })
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

  pub fn get_level_passes(&self) -> LevelPasses<'_> {
    LevelPasses {
      cull: &self.static_cull,
      gbuffer: &self.static_gbuffer,
      pyramid: &self.pyramid,
      combine: &self.combine,
    }
  }

  /// Rebuilds pipelines from a shader library reloaded since they were built.
  pub fn refresh(&mut self, shaders: &ShaderLibrary) {
    let device: &wgpu::Device = &self.context.device;

    for grid in self.grids.values_mut() {
      grid.refresh(device, shaders);
    }

    self.static_cull.refresh(device, shaders);
    self.static_gbuffer.refresh(device, shaders);
    self.pyramid.refresh(device, shaders);
    self.combine.refresh(shaders);
  }
}
