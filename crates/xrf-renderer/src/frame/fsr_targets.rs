/// What FSR 2 keeps for one viewport: its working targets at the drawn size, the reconstructed depth, and two frames of
/// its own targets, one written while the other is read, swapped every frame.
pub struct FsrTargets {
  /// The drawn size, then the viewport's.
  pub render: (u32, u32),
  pub display: (u32, u32),
  /// The frame before the blended surfaces drew, which the reactive mask compares with.
  pub opaque_texture: wgpu::Texture,
  pub opaque: wgpu::TextureView,
  /// The mean log luma an eighth a side, then a 32nd.
  pub luma_first: wgpu::TextureView,
  pub luma_shading: wgpu::TextureView,
  pub reactive: wgpu::TextureView,
  /// YCoCg with the depth clip, and the reactive and accumulation masks.
  pub prepared: wgpu::TextureView,
  pub masks: wgpu::TextureView,
  /// Whether each drawn texel is a thin feature to lock.
  pub locks: wgpu::TextureView,
  /// The depth of the frame before as each texel's nearest depth reprojected puts it, a depth's bits a drawn texel.
  pub reconstructed: wgpu::Buffer,
  /// Each frame's nearest depth, the motion found there, and the luma the locks read.
  pub dilated_depth: [wgpu::TextureView; 2],
  pub dilated_motion: [wgpu::TextureView; 2],
  pub lock_luma: [wgpu::TextureView; 2],
  /// Each frame's resolved colour with its temporal reactivity, its locks, and its four last lumas, at the viewport's
  /// size.
  pub history_textures: [wgpu::Texture; 2],
  pub history: [wgpu::TextureView; 2],
  pub lock_status: [wgpu::TextureView; 2],
  pub luma_history: [wgpu::TextureView; 2],
  /// The frame this one writes.
  pub index: usize,
  /// Frames resolved since the histories were made: FSR's `FrameIndex`.
  pub frame_index: u32,
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
    let create_texture = |label: &str, format: wgpu::TextureFormat, (width, height): (u32, u32), extra| {
      device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
          width: width.max(1),
          height: height.max(1),
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | extra,
        view_formats: &[],
      })
    };
    let create = |label: &str, format: wgpu::TextureFormat, size: (u32, u32)| {
      create_texture(label, format, size, wgpu::TextureUsages::empty()).create_view(&Default::default())
    };
    let pair = |label: &str, format: wgpu::TextureFormat, size: (u32, u32)| {
      [create(label, format, size), create(label, format, size)]
    };
    let opaque_texture: wgpu::Texture =
      create_texture("fsr2 opaque", opaque_format, render, wgpu::TextureUsages::COPY_DST);
    let history_textures: [wgpu::Texture; 2] = ["fsr2 history 0", "fsr2 history 1"]
      .map(|label| create_texture(label, Self::HISTORY, display, wgpu::TextureUsages::COPY_SRC));

    Self {
      render,
      display,
      opaque: opaque_texture.create_view(&Default::default()),
      opaque_texture,
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
      dilated_depth: pair("fsr2 dilated depth", Self::DEPTH, render),
      dilated_motion: pair("fsr2 dilated motion", Self::PAIR, render),
      lock_luma: pair("fsr2 lock luma", Self::LUMA, render),
      history: [
        history_textures[0].create_view(&Default::default()),
        history_textures[1].create_view(&Default::default()),
      ],
      history_textures,
      lock_status: pair("fsr2 lock status", Self::PAIR, display),
      luma_history: pair("fsr2 luma history", Self::LUMA_HISTORY, display),
      index: 0,
      frame_index: 0,
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

  /// The written frame becomes the one read.
  pub fn swap(&mut self) {
    self.index = 1 - self.index;
    self.frame_index += 1;
  }
}
