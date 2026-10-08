use xrf_error::{XrfError, XrfResult};

/// Runs pipeline creation under a validation scope, so a rejected pipeline is an error rather than a later failure.
pub fn create_checked<T>(device: &wgpu::Device, label: &str, create: impl FnOnce() -> T) -> XrfResult<T> {
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let created: T = create();

  match pollster::block_on(scope.pop()) {
    None => Ok(created),
    Some(error) => Err(XrfError::new_invalid_error(format!("Pipeline '{label}': {error}"))),
  }
}
