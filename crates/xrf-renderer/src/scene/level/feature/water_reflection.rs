use crate::frame::frame_texture::FrameTexture;
use crate::frame::history::History;
use crate::frame::view_targets::ViewTargets;
use crate::pass::trace_request_list::TraceRequestList;

/// The enhanced water's reflection: two histories at the drawn size, one written each frame over the other it reads,
/// and the list its rays the screen does not answer are left to the scene pass in, an eighth of the drawn pixels while
/// it traces, a stub otherwise.
pub struct WaterReflection {
  pub histories: History<FrameTexture>,
  pub requests: TraceRequestList,
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
  /// How many times its turns a still, dim reflection waits between its rays under the reflections' ray budgets, as the
  /// surfaces' dim tier does.
  pub const DIM_TURNS: u32 = 4;

  pub fn new(device: &wgpu::Device, targets: &ViewTargets) -> Self {
    let size: (u32, u32) = (targets.width, targets.height);

    Self {
      histories: History::new(|side| {
        FrameTexture::new(
          device,
          ["water reflection 0", "water reflection 1"][side],
          size,
          Self::FORMAT,
          wgpu::TextureUsages::empty(),
        )
      }),
      requests: TraceRequestList::new(device, "water reflection", 0),
      size,
    }
  }

  /// Bytes its histories and its request list take.
  pub fn get_memory(&self) -> u64 {
    let histories: u64 = self.histories.get_sides().iter().map(|side| side.get_memory()).sum();

    histories + self.requests.get_memory()
  }
}
