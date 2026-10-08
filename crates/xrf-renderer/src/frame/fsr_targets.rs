use crate::frame::frame_texture::FrameTexture;
use crate::frame::fsr_frame::FsrFrame;
use crate::frame::history::History;

/// What FSR 2 keeps for one viewport: its working targets at the drawn size, the reconstructed depth, and two frames of
/// its own targets, one written while the other is read, swapped every frame.
pub struct FsrTargets {
  /// The drawn size, then the viewport's.
  pub render: (u32, u32),
  pub display: (u32, u32),
  /// The frame before the blended surfaces drew, which the reactive mask compares with.
  pub opaque: FrameTexture,
  /// The mean log luma an eighth a side, then a 32nd.
  pub luma_first: FrameTexture,
  pub luma_shading: FrameTexture,
  pub reactive: FrameTexture,
  /// YCoCg with the depth clip, and the reactive and accumulation masks.
  pub prepared: FrameTexture,
  pub masks: FrameTexture,
  /// Whether each drawn texel is a thin feature to lock.
  pub locks: FrameTexture,
  /// The depth of the frame before as each texel's nearest depth reprojected puts it, a depth's bits a drawn texel.
  pub reconstructed: wgpu::Buffer,
  /// What each frame writes for the next; its frames swapped are FSR's `FrameIndex`.
  pub frames: History<FsrFrame>,
}

impl FsrTargets {
  pub const HISTORY: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const LUMA: wgpu::TextureFormat = wgpu::TextureFormat::R16Float;
  pub const MASK: wgpu::TextureFormat = wgpu::TextureFormat::R8Unorm;
  pub const PREPARED: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const PAIR: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
  pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::R32Float;
  pub const LUMA_HISTORY: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  /// Drawn texels a side of the first luma step.
  pub const LUMA_FIRST_STEP: u32 = 8;
  /// Drawn texels a side of the shading change mip: `2 << FFX_FSR2_SHADING_CHANGE_MIP_LEVEL`.
  pub const SHADING_CHANGE_DIVISOR: u32 = 32;

  pub fn new(
    device: &wgpu::Device,
    render: (u32, u32),
    display: (u32, u32),
    opaque_format: wgpu::TextureFormat,
  ) -> Self {
    let create = |label: &str, format: wgpu::TextureFormat, size: (u32, u32)| {
      FrameTexture::new(device, label, size, format, wgpu::TextureUsages::empty())
    };

    Self {
      render,
      display,
      opaque: FrameTexture::new(
        device,
        "fsr2 opaque",
        render,
        opaque_format,
        wgpu::TextureUsages::COPY_DST,
      ),
      luma_first: create(
        "fsr2 luma first",
        Self::LUMA,
        (
          render.0.div_ceil(Self::LUMA_FIRST_STEP),
          render.1.div_ceil(Self::LUMA_FIRST_STEP),
        ),
      ),
      luma_shading: create("fsr2 luma shading", Self::LUMA, Self::get_luma_mip_size(render)),
      reactive: create("fsr2 reactive", Self::MASK, render),
      prepared: create("fsr2 prepared", Self::PREPARED, render),
      masks: create("fsr2 masks", Self::PAIR, render),
      locks: create("fsr2 locks", Self::MASK, render),
      reconstructed: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("fsr2 reconstructed depth"),
        size: (u64::from(render.0.max(1)) * u64::from(render.1.max(1)) * 4).max(4),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      frames: History::new(|side| FsrFrame {
        dilated_depth: create("fsr2 dilated depth", Self::DEPTH, render),
        dilated_motion: create("fsr2 dilated motion", Self::PAIR, render),
        lock_luma: create("fsr2 lock luma", Self::LUMA, render),
        history: FrameTexture::new(
          device,
          ["fsr2 history 0", "fsr2 history 1"][side],
          display,
          Self::HISTORY,
          wgpu::TextureUsages::COPY_SRC,
        ),
        lock_status: create("fsr2 lock status", Self::PAIR, display),
        luma_history: create("fsr2 luma history", Self::LUMA_HISTORY, display),
      }),
    }
  }

  /// The shading change mip's size for a drawing: `iLumaMipDimensions`.
  pub fn get_luma_mip_size((width, height): (u32, u32)) -> (u32, u32) {
    (
      (width / Self::SHADING_CHANGE_DIVISOR).max(1),
      (height / Self::SHADING_CHANGE_DIVISOR).max(1),
    )
  }

  pub fn is_sized(&self, render: (u32, u32), display: (u32, u32)) -> bool {
    self.render == render && self.display == display
  }

  /// Bytes its targets, both frames and the reconstructed depth take.
  pub fn get_memory(&self) -> u64 {
    let singles: u64 = [
      &self.opaque,
      &self.luma_first,
      &self.luma_shading,
      &self.reactive,
      &self.prepared,
      &self.masks,
      &self.locks,
    ]
    .iter()
    .map(|texture| texture.get_memory())
    .sum();
    let frames: u64 = self.frames.get_sides().iter().map(|frame| frame.get_memory()).sum();

    singles + frames + self.reconstructed.size()
  }
}
