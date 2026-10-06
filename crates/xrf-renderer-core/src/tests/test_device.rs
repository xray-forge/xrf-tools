/// A device to execute graphs on: hardware where there is one, the software adapter (WARP under D3D12) otherwise, or
/// `None`, said aloud, on a machine with neither.
pub fn create_device() -> Option<(wgpu::Device, wgpu::Queue)> {
  for backends in [wgpu::Backends::DX12, wgpu::Backends::VULKAN, wgpu::Backends::PRIMARY] {
    let mut descriptor: wgpu::InstanceDescriptor = wgpu::InstanceDescriptor::new_without_display_handle();

    descriptor.backends = backends;

    let instance: wgpu::Instance = wgpu::Instance::new(descriptor);

    for is_fallback in [false, true] {
      let Ok(adapter) = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: is_fallback,
        apply_limit_buckets: false,
      })) else {
        continue;
      };

      // Timestamps where the adapter writes them, so the graph's timer has something to time.
      let features: wgpu::Features =
        adapter.features() & (wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS);

      if let Ok(pair) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: features,
        ..Default::default()
      })) {
        return Some(pair);
      }
    }
  }

  eprintln!("Skipped: no GPU to execute the graph on");

  None
}
