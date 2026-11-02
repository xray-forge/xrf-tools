use wgpu::util::DeviceExt;

/// One vertex buffer of a view's helpers, its wires' ends or its icons, and how many items it holds.
pub struct HelperBuffer {
  pub buffer: wgpu::Buffer,
  pub count: u32,
}

impl HelperBuffer {
  /// A buffer of `items` named `label`, none for none.
  pub fn new<T: bytemuck::Pod>(device: &wgpu::Device, label: &str, items: &[T]) -> Option<Self> {
    (!items.is_empty()).then(|| Self {
      buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(items),
        usage: wgpu::BufferUsages::VERTEX,
      }),
      count: items.len() as u32,
    })
  }
}
