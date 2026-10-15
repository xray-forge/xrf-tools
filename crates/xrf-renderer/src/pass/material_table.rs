use wgpu::util::DeviceExt;
use xrf_math::EPS_S;

/// The engine's material table (`r2_material`), uploaded once and sampled by every pass that lights a surface.
pub struct MaterialTable {
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
}

impl MaterialTable {
  /// Columns: `dot(L, N)`, nearly linear, so few.
  pub const LUT_WIDTH: u32 = 128;

  /// Rows: `dot(H, N)`, the specular lobe, so many.
  pub const LUT_HEIGHT: u32 = 256;

  /// Slices: the four lighting models a texture descriptor chooses between.
  pub const LUT_DEPTH: u32 = 4;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let view: wgpu::TextureView = device
      .create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
          label: Some("material table"),
          size: wgpu::Extent3d {
            width: Self::LUT_WIDTH,
            height: Self::LUT_HEIGHT,
            depth_or_array_layers: Self::LUT_DEPTH,
          },
          mip_level_count: 1,
          sample_count: 1,
          dimension: wgpu::TextureDimension::D3,
          format: wgpu::TextureFormat::Rg8Unorm,
          usage: wgpu::TextureUsages::TEXTURE_BINDING,
          view_formats: &[],
        },
        Default::default(),
        &Self::create_lut(),
      )
      .create_view(&Default::default());
    let sampler: wgpu::Sampler = device.create_sampler(&wgpu::SamplerDescriptor {
      label: Some("material table"),
      // `smp_material`: a slice past the last wraps to the first, as a texture's material can pass it.
      address_mode_w: wgpu::AddressMode::Repeat,
      mag_filter: wgpu::FilterMode::Linear,
      min_filter: wgpu::FilterMode::Linear,
      ..Default::default()
    });

    Self { view, sampler }
  }

  /// The engine's `$user$material` lookup, built as `r4_rendertarget_build_textures.cpp` builds it.
  ///
  /// `Rg8Unorm` texels, slice by slice, row by row, diffuse in red and specular in green.
  pub fn create_lut() -> Vec<u8> {
    let mut data: Vec<u8> = Vec::with_capacity((Self::LUT_WIDTH * Self::LUT_HEIGHT * Self::LUT_DEPTH * 2) as usize);

    for slice in 0..Self::LUT_DEPTH {
      for y in 0..Self::LUT_HEIGHT {
        for x in 0..Self::LUT_WIDTH {
          let is_corner: bool = y == Self::LUT_HEIGHT - 1 && x == Self::LUT_WIDTH - 1;
          let texel: [u8; 2] = if is_corner {
            [255, 255]
          } else {
            Self::sample_material(slice, x, y)
          };

          data.extend_from_slice(&texel);
        }
      }
    }

    data
  }

  /// One texel of one lighting model, as the engine quantises it.
  ///
  /// Double precision keeps the table byte for byte the one the web viewer uploads.
  fn sample_material(slice: u32, x: u32, y: u32) -> [u8; 2] {
    let ld: f64 = f64::from(x) / f64::from(Self::LUT_WIDTH - 1);
    let ls: f64 = (f64::from(y) / f64::from(Self::LUT_HEIGHT - 1) + f64::from(EPS_S)) * ld.powf(1.0 / 32.0);

    let (diffuse, specular): (f64, f64) = match slice {
      // Looks like Oren-Nayar.
      0 => (ld.powf(0.75), ls.powf(16.0) * 0.5),
      // Looks like Blinn.
      1 => (ld.powf(0.9), ls.powf(24.0)),
      // Looks like Phong.
      2 => (ld, (ls * 1.01).powf(128.0)),
      // Looks like metal.
      _ => {
        let s0: f64 = (1.0 - (0.05 * (33.0 * ld).sin() + ld - ls).abs()).abs();
        let s1: f64 = (1.0 - (0.05 * (33.0 * ld * ls).cos() + ld - ls).abs()).abs();
        let s2: f64 = (1.0 - (ld - ls).abs()).abs();

        (ld, s0.max(s1).max(s2).powf(24.0) * ld.powf(1.0 / 7.0))
      }
    };

    [Self::quantise(diffuse), Self::quantise(specular)]
  }

  /// `clampr(iFloor(value * 255.5f), 0, 255)`.
  fn quantise(value: f64) -> u8 {
    (value * 255.5).floor().clamp(0.0, 255.0) as u8
  }
}
