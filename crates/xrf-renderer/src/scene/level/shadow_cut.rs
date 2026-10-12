use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;

/// One cull and draw of a shadow's map, or of a region of it: the visible lists and draw arguments its cull fills, and
/// the camera it is culled and drawn by.
pub struct ShadowCut {
  pub lists: GrowableBuffer,
  pub args: wgpu::Buffer,
  pub view: ViewBinding,
}

impl ShadowCut {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, label: &'static str, args_size: u64) -> Self {
    Self {
      lists: GrowableBuffer::new(device, label, wgpu::BufferUsages::STORAGE),
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: args_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      view: ViewBinding::new(device, view_layout),
    }
  }
}
