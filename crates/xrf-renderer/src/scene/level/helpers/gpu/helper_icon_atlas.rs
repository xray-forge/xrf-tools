use image::RgbaImage;
use image::imageops::FilterType;
use xrf_error::{XrfError, XrfResult};

use crate::contract::render_helper_icon::RENDER_HELPER_ICONS;
use crate::host::render_bundle::RenderBundle;

/// The helpers' icons as one texture: a cell each, [`RENDER_HELPER_ICONS`] row by row, white pictures outlined in
/// black that a helper tints by its kind's colour, with mips down to an eighth of a cell.
pub struct HelperIconAtlas {
  pub view: wgpu::TextureView,
  pub sampler: wgpu::Sampler,
}

impl HelperIconAtlas {
  /// The atlas's path in the renderer's bundle, which the generator writes (`cargo make generate-helper-icons`).
  pub const PATH: &'static str = "helpers/icons.png";

  /// Pixels a cell spans each way, cells a row, and the pixels a cell's picture leaves clear round it, so neither its
  /// outline nor a smaller mip reaches the next cell.
  pub const CELL: u32 = 64;
  pub const COLUMNS: u32 = 8;
  pub const MARGIN: u32 = 8;

  /// Mips it keeps, the smallest a cell of eight pixels.
  const MIPS: u32 = 4;

  /// The atlas's size in pixels for `count` icons.
  pub fn get_size(count: usize) -> (u32, u32) {
    (
      Self::COLUMNS * Self::CELL,
      (count as u32).div_ceil(Self::COLUMNS) * Self::CELL,
    )
  }

  /// The atlas read from `bundle`, with its mips.
  ///
  /// # Errors
  ///
  /// Returns an error where the bundle does not hold it, it does not decode, or it is not the size its icons make.
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, bundle: &dyn RenderBundle) -> XrfResult<Self> {
    let image: RgbaImage =
      image::load_from_memory_with_format(&bundle.read_bundled(Self::PATH)?, image::ImageFormat::Png)
        .map_err(|error| XrfError::new_unexpected_error(format!("The helper icon atlas does not decode: {error}")))?
        .to_rgba8();
    let (width, height) = Self::get_size(RENDER_HELPER_ICONS.len());

    if image.dimensions() != (width, height) {
      return Err(XrfError::new_unexpected_error(format!(
        "The helper icon atlas is {:?} pixels, not the {width}x{height} its icons take; generate it again",
        image.dimensions()
      )));
    }

    let texture: wgpu::Texture = device.create_texture(&wgpu::TextureDescriptor {
      label: Some("helper icon atlas"),
      size: wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
      mip_level_count: Self::MIPS,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8Unorm,
      usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
      view_formats: &[],
    });
    let mut level: RgbaImage = image;

    for mip in 0..Self::MIPS {
      if mip > 0 {
        level = image::imageops::resize(&level, level.width() / 2, level.height() / 2, FilterType::Triangle);
      }

      queue.write_texture(
        wgpu::TexelCopyTextureInfo {
          texture: &texture,
          mip_level: mip,
          origin: wgpu::Origin3d::ZERO,
          aspect: wgpu::TextureAspect::All,
        },
        level.as_raw(),
        wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(level.width() * 4),
          rows_per_image: Some(level.height()),
        },
        wgpu::Extent3d {
          width: level.width(),
          height: level.height(),
          depth_or_array_layers: 1,
        },
      );
    }

    Ok(Self {
      view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("helper icon atlas"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::MipmapFilterMode::Linear,
        ..Default::default()
      }),
    })
  }
}
