use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::frame_texture::FrameTexture;
use crate::frame::history::History;
use crate::frame::reflection_frame::ReflectionFrame;
use crate::frame::view_targets::ViewTargets;
use crate::pass::trace_request_list::TraceRequestList;

/// What the screen-space reflections keep from one frame to the next: what each frame writes for the next, a side
/// read while the other is written; at the frame's size, the frame's depth reduced to its nearest, which each frame
/// rebuilds from the depth as it stood before the grass drew, and the last frame's scene as it stood before the sun
/// shafts and the flares, with the depth target it was drawn over, which a hit reads its colour from; and the rays the
/// trace leaves to the scene pass.
pub struct ReflectionHistory {
  pub frames: History<ReflectionFrame>,
  /// The size it traces at.
  pub size: (u32, u32),
  pub pyramid: DepthPyramid,
  pub scene: FrameTexture,
  /// The view's depth target the last frame drew into, which it keeps while the view draws into the other.
  pub depth: wgpu::TextureView,
  /// Whether the scene and depth kept hold the last frame.
  pub is_source_valid: bool,
  /// The rays the trace leaves to the scene pass each frame, a quarter of the traced pixels at most while it traces, a
  /// stub otherwise.
  pub requests: TraceRequestList,
}

impl ReflectionHistory {
  pub const RADIANCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const HELD_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
  /// Levels of the nearest depth's pyramid a ray climbs through: the depth itself and six halvings.
  pub const PYRAMID_LEVELS: u32 = 7;
  /// Traced pixels a request the list holds stands for at most: a ray more than that leaves its pixel the sky.
  pub const REQUEST_SHARE: u32 = 4;

  /// The history at the size traced at, and the pyramid and the kept scene over the frame's size.
  pub fn new(device: &wgpu::Device, size: (u32, u32), frame_size: (u32, u32)) -> Self {
    let create = |label: &str, size: (u32, u32), format: wgpu::TextureFormat, usage: wgpu::TextureUsages| {
      FrameTexture::new(device, label, size, format, usage)
    };
    let none: wgpu::TextureUsages = wgpu::TextureUsages::empty();
    let copied: wgpu::TextureUsages = wgpu::TextureUsages::COPY_DST;

    Self {
      frames: History::new(|side| ReflectionFrame {
        radiance: create(
          ["reflection history 0", "reflection history 1"][side],
          size,
          Self::RADIANCE_FORMAT,
          none,
        ),
        surface: create(
          ["reflection history surface 0", "reflection history surface 1"][side],
          size,
          Self::SURFACE_FORMAT,
          none,
        ),
        held: create(
          ["reflection history held 0", "reflection history held 1"][side],
          size,
          Self::HELD_FORMAT,
          none,
        ),
        full: create(
          ["reflection history full 0", "reflection history full 1"][side],
          frame_size,
          Self::RADIANCE_FORMAT,
          none,
        ),
      }),
      size,
      pyramid: DepthPyramid::new_nearest(device, frame_size.0, frame_size.1, Self::PYRAMID_LEVELS),
      scene: create("reflection source scene", frame_size, ViewTargets::SCENE, copied),
      depth: ViewTargets::create_depth(device, frame_size),
      is_source_valid: false,
      requests: TraceRequestList::new(device, "reflection trace", 0),
    }
  }

  /// Whether it is traced at a size over a frame of another.
  pub fn is_sized(&self, size: (u32, u32), frame_size: (u32, u32)) -> bool {
    self.size == size && (self.pyramid.width, self.pyramid.height) == frame_size
  }

  /// The written set becomes the one read, the scene copied this frame the one the next reads, and the depth target
  /// this frame drew into (`depth`, the view's) the one it keeps, the view drawing into the other next.
  pub fn swap(&mut self, depth: &mut wgpu::TextureView) {
    self.frames.swap();
    std::mem::swap(&mut self.depth, depth);
    self.is_source_valid = true;
  }

  /// Bytes its textures and the pyramid take.
  pub fn get_memory(&self) -> u64 {
    let frames: u64 = self.frames.get_sides().iter().map(|frame| frame.get_memory()).sum();
    let (width, height) = self.scene.get_size();
    let depth: u64 = u64::from(width) * u64::from(height) * 4;

    frames + self.scene.get_memory() + depth + self.pyramid.get_memory()
  }
}
