use xrf_error::XrfResult;
use xrf_renderer_core::{ComputePipelineDescription, PipelineCache, RenderPipelineDescription, ShaderSource};

use crate::shader::shader_library::ShaderLibrary;

/// Where a feature unit asks for its pipelines: the renderer's cache, making what it lacks on the device from the
/// shader library, each module composed for the device's features.
#[derive(Clone, Copy)]
pub struct PipelineSource<'a> {
  pub cache: &'a PipelineCache,
  pub device: &'a wgpu::Device,
  pub shaders: &'a ShaderLibrary,
}

impl<'a> PipelineSource<'a> {
  /// How many invalidations the cache has had, which a unit keeping its pipelines compares to ask again.
  pub fn get_generation(&self) -> u64 {
    self.cache.get_generation()
  }

  /// The render pipeline `description` describes, made once.
  ///
  /// # Errors
  ///
  /// Returns an error when the module cannot be composed or wgpu refuses the module or the pipeline.
  pub fn get_render_pipeline(&self, description: &RenderPipelineDescription) -> XrfResult<wgpu::RenderPipeline> {
    self.cache.get_render_pipeline(self.device, self, description)
  }

  /// The compute pipeline `description` describes, made once.
  ///
  /// # Errors
  ///
  /// Returns an error when the module cannot be composed or wgpu refuses the module or the pipeline.
  pub fn get_compute_pipeline(&self, description: &ComputePipelineDescription) -> XrfResult<wgpu::ComputePipeline> {
    self.cache.get_compute_pipeline(self.device, self, description)
  }

  /// The shader module `module` names, composed for the device's features and compiled once.
  ///
  /// # Errors
  ///
  /// Returns an error when the module cannot be composed or wgpu refuses it.
  pub fn get_module(&self, module: &'static str) -> XrfResult<wgpu::ShaderModule> {
    self.cache.get_module(self.device, self, module)
  }

  /// The driver's cache a pipeline made by hand is made with, none where the device keeps none.
  pub fn get_driver_cache(&self) -> Option<&'a wgpu::PipelineCache> {
    self.cache.get_driver_cache()
  }

  /// Makes the render pipelines described ahead of the frames that draw with them.
  ///
  /// # Errors
  ///
  /// Returns the first pipeline's error.
  pub fn warm(&self, render: &[RenderPipelineDescription]) -> XrfResult {
    self.cache.warm(self.device, self, render, &[])
  }
}

impl ShaderSource for PipelineSource<'_> {
  fn compose(&self, module: &str) -> XrfResult<String> {
    self.shaders.compose_for(module, self.device.features())
  }
}
