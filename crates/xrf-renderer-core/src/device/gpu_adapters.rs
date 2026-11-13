use crate::device::render_gpu::RenderGpu;

/// How a GPU's adapter is chosen, the same for the renderer's device and the ray query device: the hardware adapter a
/// setting names where it can do the work, else the one the instance picks itself.
pub struct GpuAdapters;

impl GpuAdapters {
  /// The hardware adapter of `backends` on `instance` that runs on `gpu` and that `accepts` takes, if any.
  pub fn find(
    instance: &wgpu::Instance,
    backends: wgpu::Backends,
    gpu: RenderGpu,
    accepts: impl Fn(&wgpu::Adapter) -> bool,
  ) -> Option<wgpu::Adapter> {
    pollster::block_on(instance.enumerate_adapters(backends))
      .into_iter()
      .find(|adapter| {
        let info: wgpu::AdapterInfo = adapter.get_info();

        info.device_type != wgpu::DeviceType::Cpu && RenderGpu::of(&info) == gpu && accepts(adapter)
      })
  }

  /// The adapter `instance` picks itself: its high-performance one, or its software one where `is_software`.
  ///
  /// # Errors
  ///
  /// Returns wgpu's error when the instance has no such adapter.
  pub fn request_automatic(
    instance: &wgpu::Instance,
    is_software: bool,
  ) -> Result<wgpu::Adapter, wgpu::RequestAdapterError> {
    pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::HighPerformance,
      compatible_surface: None,
      force_fallback_adapter: is_software,
      apply_limit_buckets: false,
    }))
  }
}
