/// One viewport's G-buffer, depth, light and scene, sized to its rectangle and made again when that changes; the targets
/// each frame writes afresh (the ambient occlusion, the haze, the distortion, the high part and the bloom) are its
/// graph's, their formats and sizes here.
pub struct ViewTargets {
  pub width: u32,
  pub height: u32,
  /// Albedo, then gloss.
  pub albedo: wgpu::TextureView,
  /// The view space normal, octahedral.
  pub normal: wgpu::TextureView,
  /// Hemisphere, sun, material slice.
  pub material: wgpu::TextureView,
  /// How far each surface's point moved on the screen since the last frame: texture coordinates, now less then, `y`
  /// down; zero where nothing was drawn.
  pub motion: wgpu::TextureView,
  /// Reversed: one at the near plane, zero where nothing was drawn.
  pub depth: wgpu::TextureView,
  /// What the lights accumulate: diffuse in colour, specular in alpha.
  pub light: wgpu::TextureView,
  /// The scene combine finished, tonemapped, before it is put into the window.
  pub scene: wgpu::TextureView,
  /// The scene's texture, which a temporal resolve copies the frame it resolved back into, and FSR 2 copies out of
  /// before the blended surfaces draw.
  pub scene_texture: wgpu::Texture,
}

impl ViewTargets {
  pub const ALBEDO: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  pub const NORMAL: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
  pub const MATERIAL: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  pub const MOTION: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;
  pub const DEPTH: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
  pub const LIGHT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const SCENE: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const OCCLUSION: wgpu::TextureFormat = wgpu::TextureFormat::Rg32Float;
  pub const HAZE: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const DISTORTION: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  /// Eight bits a channel, as the engine's `A8R8G8B8` targets, so each stage of the bloom clamps to one as there.
  pub const HIGH: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  pub const BLOOM: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
  /// `BLOOM_size_X` and `BLOOM_size_Y`, whatever the frame's size.
  pub const BLOOM_SIZE: (u32, u32) = (256, 256);
  /// What the distortion target holds where nothing distorts it, `(127, 127, 0, 127)` as the engine clears it.
  pub const NEUTRAL_DISTORTION: f64 = 127.0 / 255.0;
  /// The distortion target cleared to nothing distorting, as each frame starts it.
  pub const DISTORTION_CLEAR: wgpu::Color = wgpu::Color {
    r: Self::NEUTRAL_DISTORTION,
    g: Self::NEUTRAL_DISTORTION,
    b: 0.0,
    a: Self::NEUTRAL_DISTORTION,
  };
  /// Texels the haze map holds across, one a bearing, and down, one a height, as `shaders/frame/sky_haze.wgsl` says.
  pub const HAZE_SIZE: (u32, u32) = (64, 32);

  /// A depth target of the view's, which the reflections keep a frame's of in turn.
  pub fn create_depth(device: &wgpu::Device, (width, height): (u32, u32)) -> wgpu::TextureView {
    device
      .create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
          width,
          height,
          depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: Self::DEPTH,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
      })
      .create_view(&Default::default())
  }

  pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
    let create_sized = |label: &str, format: wgpu::TextureFormat, (width, height): (u32, u32)| -> wgpu::TextureView {
      device
        .create_texture(&wgpu::TextureDescriptor {
          label: Some(label),
          size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D2,
          format,
          usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        })
        .create_view(&Default::default())
    };
    let create =
      |label: &str, format: wgpu::TextureFormat| -> wgpu::TextureView { create_sized(label, format, (width, height)) };
    let depth: wgpu::TextureView = Self::create_depth(device, (width, height));
    let scene_texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("scene"),
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: Self::SCENE,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT
        | wgpu::TextureUsages::TEXTURE_BINDING
        | wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });

    Self {
      width,
      height,
      albedo: create("albedo", Self::ALBEDO),
      normal: create("normal", Self::NORMAL),
      material: create("material", Self::MATERIAL),
      motion: create("motion", Self::MOTION),
      depth,
      light: create("light", Self::LIGHT),
      scene: scene_texture.create_view(&Default::default()),
      scene_texture,
    }
  }

  pub fn is_sized(&self, width: u32, height: u32) -> bool {
    self.width == width && self.height == height
  }
}
