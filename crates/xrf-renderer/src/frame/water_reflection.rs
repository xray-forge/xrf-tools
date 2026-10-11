use crate::frame::view_targets::ViewTargets;
use crate::pass::trace_request_list::TraceRequestList;

/// The enhanced water's reflection: two histories at the drawn size, one written each frame over the other it reads,
/// and the list its rays the screen does not answer are left to the scene pass in, an eighth of the drawn pixels while
/// it traces, a stub otherwise.
pub struct WaterReflection {
  pub histories: [wgpu::TextureView; 2],
  pub requests: TraceRequestList,
  /// The history this frame writes.
  pub index: usize,
  /// Whether the history this frame reads holds a frame.
  pub is_valid: bool,
  /// The size it was made at, the targets' drawn size.
  pub size: (u32, u32),
}

impl WaterReflection {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// This frame's reflection before the history keeps it: its colour, and the distance along the view of what it met.
  pub const CURRENT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// Drawn pixels a request the list holds stands for at most while it traces.
  pub const REQUEST_SHARE: u32 = 8;
  /// Frames a pixel waits between its rays at the least, the history held between: the water's reflection keeps 0.97
  /// of the last frame's and is blurred, so a pixel traced once in eight shows no difference and stays within its list.
  pub const TRACE_TURNS_LEAST: u32 = 8;

  pub fn new(device: &wgpu::Device, targets: &ViewTargets) -> Self {
    let create = |label: &str| -> wgpu::TextureView {
      device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some(label),
          size: wgpu::Extent3d {
            width: targets.width,
            height: targets.height,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format: Self::FORMAT,
          usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default())
    };

    Self {
      histories: [create("water reflection 0"), create("water reflection 1")],
      requests: TraceRequestList::new(device, "water reflection", 0),
      index: 0,
      is_valid: false,
      size: (targets.width, targets.height),
    }
  }

  /// Makes the history just written the one the next frame reads.
  pub fn swap(&mut self) {
    self.index = 1 - self.index;
    self.is_valid = true;
  }
}
