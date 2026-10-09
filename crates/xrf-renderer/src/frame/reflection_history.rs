use crate::frame::depth_pyramid::DepthPyramid;

/// What the screen-space reflections keep from one frame to the next, at the size they trace at: the reflection and
/// its variance, the surface's world normal and roughness, and the distance along the view of the point, how many
/// frames its reflection holds and the distance through it to what it reflects; one set read while the other is written, swapped every frame. Beside them, the frame's
/// depth reduced to its nearest, which each frame rebuilds.
pub struct ReflectionHistory {
  pub radiances: [wgpu::Texture; 2],
  pub radiance_views: [wgpu::TextureView; 2],
  pub surfaces: [wgpu::Texture; 2],
  pub surface_views: [wgpu::TextureView; 2],
  pub helds: [wgpu::Texture; 2],
  pub held_views: [wgpu::TextureView; 2],
  pub pyramid: DepthPyramid,
  /// The set this frame writes.
  pub index: usize,
  /// Whether the set this frame reads holds a frame.
  pub is_valid: bool,
  /// Frames traced since it was made, which turns each pixel's noise.
  pub frame: u32,
}

impl ReflectionHistory {
  pub const RADIANCE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const SURFACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
  pub const HELD_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba32Float;
  /// Levels of the nearest depth's pyramid a ray climbs through: the depth itself and six halvings.
  pub const PYRAMID_LEVELS: u32 = 7;

  /// The history at the size traced at, and the pyramid over the frame's depth.
  pub fn new(device: &wgpu::Device, (width, height): (u32, u32), (frame_width, frame_height): (u32, u32)) -> Self {
    let create = |label: &str, format: wgpu::TextureFormat| -> (wgpu::Texture, wgpu::TextureView) {
      let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
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
      });
      let view: wgpu::TextureView = texture.create_view(&Default::default());

      (texture, view)
    };
    let pair = |labels: [&str; 2], format: wgpu::TextureFormat| {
      let [(first, first_view), (second, second_view)] = labels.map(|label| create(label, format));

      ([first, second], [first_view, second_view])
    };
    let (radiances, radiance_views) = pair(["reflection history 0", "reflection history 1"], Self::RADIANCE_FORMAT);
    let (surfaces, surface_views) = pair(
      ["reflection history surface 0", "reflection history surface 1"],
      Self::SURFACE_FORMAT,
    );
    let (helds, held_views) = pair(
      ["reflection history held 0", "reflection history held 1"],
      Self::HELD_FORMAT,
    );

    Self {
      radiances,
      radiance_views,
      surfaces,
      surface_views,
      helds,
      held_views,
      pyramid: DepthPyramid::new_nearest(device, frame_width, frame_height, Self::PYRAMID_LEVELS),
      index: 0,
      is_valid: false,
      frame: 0,
    }
  }

  /// Whether it is traced at a size over a frame of another.
  pub fn is_sized(&self, size: (u32, u32), frame_size: (u32, u32)) -> bool {
    self.get_size() == size && (self.pyramid.width, self.pyramid.height) == frame_size
  }

  /// The size it is traced at.
  pub fn get_size(&self) -> (u32, u32) {
    let size: wgpu::Extent3d = self.radiances[0].size();

    (size.width, size.height)
  }

  /// The written set becomes the one read.
  pub fn swap(&mut self) {
    self.index = 1 - self.index;
    self.is_valid = true;
    self.frame = self.frame.wrapping_add(1);
  }
}
