use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::frame_texture::FrameTexture;
use crate::frame::history::History;
use crate::frame::view_targets::ViewTargets;
use crate::pass::trace_request_list::TraceRequestList;
use crate::scene::level::feature::reflection_frame::ReflectionFrame;

/// What the screen-space reflections keep from one frame to the next: what their denoiser writes each frame for the
/// next, a side read while the other is written, none while DLSS ray reconstruction takes the trace raw; at the
/// frame's size, the frame's depth reduced to its nearest, which each frame rebuilds from the depth as it stood before
/// the grass drew, and the last frame's scene as it stood before the sun shafts and the flares, with the depth target
/// it was drawn over, which a hit reads its colour from; and the rays the trace leaves to the scene pass.
pub struct ReflectionHistory {
  pub frames: Option<History<ReflectionFrame>>,
  /// Frames swapped since it was made, which turns each pixel's noise from one frame to the next.
  pub frame: u32,
  /// The size it traces at.
  pub size: (u32, u32),
  pub pyramid: DepthPyramid,
  pub scene: FrameTexture,
  /// The view's depth target the last frame drew into, which it keeps while the view draws into the other.
  pub depth: wgpu::TextureView,
  /// Whether the scene and depth kept hold the last frame.
  pub is_source_valid: bool,
  /// The rays the trace leaves to the scene pass each frame, a share of the traced pixels at most while it traces, a
  /// stub otherwise.
  pub requests: TraceRequestList,
}

impl ReflectionHistory {
  pub const RADIANCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const HELD_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
  /// Levels of the nearest depth's pyramid a ray climbs through: the depth itself and six halvings.
  pub const PYRAMID_LEVELS: u32 = 7;
  /// Traced pixels a request the list holds stands for at most: a ray more than that leaves its pixel what it reflected
  /// the frame before, or the sky.
  pub const REQUEST_SHARE: u32 = 4;
  /// The same, traced raw: every pixel traces every frame and nothing is held, so the list takes twice the share at
  /// the least, and grows to what the frames ask (`get_raw_capacity`).
  pub const RAW_REQUEST_SHARE: u32 = 2;
  /// How much more room than a raw frame asked its list is given, and the requests it grows or shrinks by at a time.
  const RAW_REQUEST_MARGIN: f32 = 1.25;
  const RAW_REQUEST_STEP: u32 = 65_536;

  /// The room a raw trace's list of `current` takes next over `traced` pixels, its frames having last asked for
  /// `asked` rays: `RAW_REQUEST_MARGIN` more than asked, in whole steps, between its share and a ray a pixel; grown at
  /// once and shrunk only once that falls to half its room, so a frame asking a little less never makes it anew.
  /// Raw, a pixel past the list has nothing held to show, and the list fills in the order pixels are drawn: the last
  /// rows of the screen reflected the sky.
  pub fn get_raw_capacity(traced: u32, current: u32, asked: u32) -> u32 {
    let least: u32 = traced.div_ceil(Self::RAW_REQUEST_SHARE);
    let wanted: u32 = ((asked as f32 * Self::RAW_REQUEST_MARGIN) as u32)
      .div_ceil(Self::RAW_REQUEST_STEP)
      .saturating_mul(Self::RAW_REQUEST_STEP)
      .clamp(least, traced.max(least));

    if wanted > current || wanted.saturating_mul(2) <= current {
      wanted
    } else {
      current.clamp(least, traced.max(least))
    }
  }

  /// The history at the size traced at, its denoiser's frames where it is denoised, and the pyramid and the kept scene
  /// over the frame's size.
  pub fn new(device: &wgpu::Device, size: (u32, u32), frame_size: (u32, u32), is_denoised: bool) -> Self {
    let create = |label: &str, size: (u32, u32), format: wgpu::TextureFormat, usage: wgpu::TextureUsages| {
      FrameTexture::new(device, label, size, format, usage)
    };
    let none: wgpu::TextureUsages = wgpu::TextureUsages::empty();
    let copied: wgpu::TextureUsages = wgpu::TextureUsages::COPY_DST;

    Self {
      frames: is_denoised.then(|| {
        History::new(|side| ReflectionFrame {
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
        })
      }),
      frame: 0,
      size,
      pyramid: DepthPyramid::new_nearest(device, frame_size.0, frame_size.1, Self::PYRAMID_LEVELS),
      scene: create("reflection source scene", frame_size, ViewTargets::SCENE, copied),
      depth: ViewTargets::create_depth(device, frame_size),
      is_source_valid: false,
      requests: TraceRequestList::new(device, "reflection trace", 0),
    }
  }

  /// Whether it is traced at a size over a frame of another, denoised or not.
  pub fn is_sized(&self, size: (u32, u32), frame_size: (u32, u32), is_denoised: bool) -> bool {
    self.size == size && (self.pyramid.width, self.pyramid.height) == frame_size && self.frames.is_some() == is_denoised
  }

  /// Whether the denoiser's side this frame reads holds the frame before.
  pub fn has_history(&self) -> bool {
    self.frames.as_ref().is_some_and(History::is_valid)
  }

  /// The written set becomes the one read, the scene copied this frame the one the next reads, and the depth target
  /// this frame drew into (`depth`, the view's) the one it keeps, the view drawing into the other next.
  pub fn swap(&mut self, depth: &mut wgpu::TextureView) {
    if let Some(frames) = &mut self.frames {
      frames.swap();
    }

    std::mem::swap(&mut self.depth, depth);
    self.frame = self.frame.wrapping_add(1);
    self.is_source_valid = true;
  }

  /// Bytes its textures and the pyramid take.
  pub fn get_memory(&self) -> u64 {
    let frames: u64 = self
      .frames
      .iter()
      .flat_map(History::get_sides)
      .map(ReflectionFrame::get_memory)
      .sum();
    let (width, height) = self.scene.get_size();
    let depth: u64 = u64::from(width) * u64::from(height) * 4;

    frames + self.scene.get_memory() + depth + self.pyramid.get_memory()
  }
}
