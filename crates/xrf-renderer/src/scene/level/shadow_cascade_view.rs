use crate::lighting::sun_cascade::SunCascade;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::growable_buffer::GrowableBuffer;

/// One cascade of a level's sun shadow: its fit, the list and draw arguments its cull fills, the camera it is culled
/// and drawn through, and what its map was last drawn at.
pub struct ShadowCascadeView {
  pub cascade: SunCascade,
  pub lists: GrowableBuffer,
  pub args: wgpu::Buffer,
  pub view: ViewBinding,
  /// The cull's bind group, with the scene's and the lists' generations and the targets' epoch it binds.
  pub cull_group: Option<((u64, u64, u64), wgpu::BindGroup)>,
  /// The draws' bind groups, with the scene's and the lists' generations they bind.
  pub draw_groups: Option<((u64, u64), [wgpu::BindGroup; 2])>,
  /// The cascade's version and the sectors resident its map was last drawn at, or none before it was drawn.
  pub drawn: Option<(u64, usize)>,
}

impl ShadowCascadeView {
  pub fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout, args_size: u64) -> Self {
    Self {
      cascade: SunCascade::default(),
      lists: GrowableBuffer::new(device, "shadow lists", wgpu::BufferUsages::STORAGE),
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("shadow draw arguments"),
        size: args_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      view: ViewBinding::new(device, view_layout),
      cull_group: None,
      draw_groups: None,
      drawn: None,
    }
  }
}
