use glam::Vec2;

/// One frame FSR upscales: drawn `render` texels across into a viewport `display` pixels across, its texels sampled
/// `jitter` off their centres (`previous_jitter` the frame before's), its jitter's phase count, the frame's index since
/// the last reset, and whether its colour is linear light past the display's range.
#[derive(Clone, Copy, Debug, Default)]
pub struct FsrFrame {
  pub render: (u32, u32),
  pub display: (u32, u32),
  pub jitter: Vec2,
  pub previous_jitter: Vec2,
  pub phases: u32,
  /// Zero on the first frame after a reset.
  pub frame_index: u32,
  /// Whether its colour is linear light, which FSR accumulates through its reversible tonemap.
  pub is_hdr: bool,
}
