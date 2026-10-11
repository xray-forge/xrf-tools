use crate::frame::frame_texture::FrameTexture;
use crate::frame::fsr_frame::FsrFrame;
use crate::frame::history::History;
use crate::pass::fsr_uniform::FsrUniform;

/// What FSR keeps for one viewport: its working targets at the drawn size and at half of it, the reconstructed depth,
/// and two frames of its own targets, one written while the other is read, swapped every frame.
pub struct FsrTargets {
  /// The drawn size, then the viewport's.
  pub render: (u32, u32),
  pub display: (u32, u32),
  /// The frame before the blended surfaces drew, which the reactive mask compares with.
  pub opaque: FrameTexture,
  pub reactive: FrameTexture,
  /// The nearest depth about each drawn texel, and the motion found there.
  pub dilated_depth: FrameTexture,
  pub dilated_motion: FrameTexture,
  /// The farthest depth in metres, at the drawn size, then averaged at half of it.
  pub farthest_depth: FrameTexture,
  pub half_farthest_depth: FrameTexture,
  /// The shading change pyramid's three levels from half the drawn size, then the shading change at half of it.
  pub shading_pyramid: [FrameTexture; 3],
  pub shading_change: FrameTexture,
  /// The reactive, disocclusion, shading change and accumulation masks.
  pub masks: FrameTexture,
  /// How strongly each drawn texel is a thin feature to lock.
  pub new_locks: FrameTexture,
  pub luma_instability: FrameTexture,
  /// The depth of the frame before as each texel's nearest depth reprojected puts it, a depth's bits a drawn texel.
  pub reconstructed: wgpu::Buffer,
  /// What each frame writes for the next; its frames swapped are FSR's `FrameIndex`.
  pub frames: History<FsrFrame>,
}

impl FsrTargets {
  pub const HISTORY: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
  pub const MOTION: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
  pub const LUMA: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;
  pub const LUMA_HISTORY: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const MASK: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
  pub const MASKS: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  pub const PYRAMID: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
  /// What a debug view is drawn into, at the display's size, made by the frame graph while one is shown.
  pub const DEBUG: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

  pub fn new(
    device: &wgpu::Device,
    render: (u32, u32),
    display: (u32, u32),
    opaque_format: wgpu::TextureFormat,
  ) -> Self {
    let create = |label: &str, format: wgpu::TextureFormat, size: (u32, u32)| {
      FrameTexture::new(device, label, size, format, wgpu::TextureUsages::empty())
    };
    let shading: (u32, u32) = FsrUniform::get_shading_size(render);

    Self {
      render,
      display,
      opaque: FrameTexture::new(
        device,
        "fsr opaque",
        render,
        opaque_format,
        wgpu::TextureUsages::COPY_DST,
      ),
      reactive: create("fsr reactive", Self::MASK, render),
      dilated_depth: create("fsr dilated depth", Self::DEPTH, render),
      dilated_motion: create("fsr dilated motion", Self::MOTION, render),
      farthest_depth: create("fsr farthest depth", Self::LUMA, render),
      half_farthest_depth: create("fsr half farthest depth", Self::LUMA, shading),
      shading_pyramid: [0, 1, 2].map(|level| {
        create(
          "fsr shading pyramid",
          Self::PYRAMID,
          ((shading.0 >> level).max(1), (shading.1 >> level).max(1)),
        )
      }),
      shading_change: create("fsr shading change", Self::MASK, shading),
      masks: create("fsr masks", Self::MASKS, render),
      new_locks: create("fsr new locks", Self::MASK, render),
      luma_instability: create("fsr luma instability", Self::LUMA, render),
      reconstructed: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fsr reconstructed depth"),
        size: (u64::from(render.0.max(1)) * u64::from(render.1.max(1)) * 4).max(4),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      frames: History::new(|side| FsrFrame {
        luma: create("fsr luma", Self::LUMA, render),
        accumulation: create("fsr accumulation", Self::MASK, render),
        luma_history: create("fsr luma history", Self::LUMA_HISTORY, render),
        history: FrameTexture::new(
          device,
          ["fsr history 0", "fsr history 1"][side],
          display,
          Self::HISTORY,
          wgpu::TextureUsages::COPY_SRC,
        ),
      }),
    }
  }

  pub fn is_sized(&self, render: (u32, u32), display: (u32, u32)) -> bool {
    self.render == render && self.display == display
  }

  /// Bytes its targets, both frames and the reconstructed depth take.
  pub fn get_memory(&self) -> u64 {
    let singles: u64 = [
      &self.opaque,
      &self.reactive,
      &self.dilated_depth,
      &self.dilated_motion,
      &self.farthest_depth,
      &self.half_farthest_depth,
      &self.shading_change,
      &self.masks,
      &self.new_locks,
      &self.luma_instability,
    ]
    .into_iter()
    .chain(&self.shading_pyramid)
    .map(|texture| texture.get_memory())
    .sum();
    let frames: u64 = self.frames.get_sides().iter().map(|frame| frame.get_memory()).sum();

    singles + frames + self.reconstructed.size()
  }
}
