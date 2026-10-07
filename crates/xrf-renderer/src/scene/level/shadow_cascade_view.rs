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
  /// The cascade's version and what the scene held when its map was last drawn, or none before it was drawn.
  pub drawn: Option<(u64, usize)>,
  /// The sway's time its map was last drawn at.
  pub drawn_at: f32,
  /// The fit its map was last drawn at, which a fit strayed too far from draws again whatever its stagger.
  pub drawn_fit: SunCascade,
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
      drawn: None,
      drawn_at: 0.0,
      drawn_fit: SunCascade::default(),
    }
  }
}
