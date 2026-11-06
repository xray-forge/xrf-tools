use crate::feature::tracing::scene_trace::requests::trace_request_list::TraceRequestList;
use crate::frame::targets::depth_pyramid::DepthPyramid;
use crate::frame::targets::frame_texture::FrameTexture;
use crate::frame::targets::history::History;
use crate::frame::targets::view_targets::ViewTargets;

/// The enhanced water's reflection: two histories at the drawn size, one written each frame over the other it reads,
/// the list its rays the screen does not answer are left to the scene pass in, an eighth of the drawn pixels while it
/// traces, a stub otherwise, and the pyramid of the frame's nearest depth its rays walk over.
pub struct WaterReflection {
  pub histories: History<FrameTexture>,
  pub requests: TraceRequestList,
  /// The frame's depth as the water finds it, the grass in it, reduced to its nearest: its own, as the surfaces' is
  /// built before the grass draws, which the water reflects.
  pub pyramid: DepthPyramid,
  /// The size it was made at, the targets' drawn size.
  pub size: (u32, u32),
}

impl WaterReflection {
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// This frame's reflection before the history keeps it: its colour, and the distance along the view of what it met.
  pub const CURRENT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  /// Drawn pixels a request the list holds stands for at most while it traces; past its room the pixels stretch their
  /// turns by the list's wait.
  pub const REQUEST_SHARE: u32 = 8;
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
      pyramid: DepthPyramid::new_nearest(device, size.0, size.1, DepthPyramid::RAY_LEVELS),
      size,
    }
  }

  /// Bytes its histories, its request list and its pyramid take.
  pub fn get_memory(&self) -> u64 {
    let histories: u64 = self.histories.get_sides().iter().map(|side| side.get_memory()).sum();

    histories + self.requests.get_memory() + self.pyramid.get_memory()
  }
}
