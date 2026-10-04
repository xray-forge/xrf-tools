use xrf_error::{XrfError, XrfResult};

use crate::shader::shader_library::ShaderLibrary;

/// A shader module composed from the library and compiled, refused when the device rejects it rather than left to
/// fail the first draw.
pub fn create_module(device: &wgpu::Device, shaders: &ShaderLibrary, name: &str) -> XrfResult<wgpu::ShaderModule> {
  let source: String = shaders.compose(name)?;
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let module: wgpu::ShaderModule = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some(name),
    source: wgpu::ShaderSource::Wgsl(source.into()),
  });

  match pollster::block_on(scope.pop()) {
    None => Ok(module),
    Some(error) => Err(XrfError::new_invalid_error(format!("Shader '{name}': {error}"))),
  }
}

/// Runs pipeline creation under a validation scope, so a rejected pipeline is an error rather than a later failure.
pub fn create_checked<T>(device: &wgpu::Device, label: &str, create: impl FnOnce() -> T) -> XrfResult<T> {
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let created: T = create();

  match pollster::block_on(scope.pop()) {
    None => Ok(created),
    Some(error) => Err(XrfError::new_invalid_error(format!("Pipeline '{label}': {error}"))),
  }
}
