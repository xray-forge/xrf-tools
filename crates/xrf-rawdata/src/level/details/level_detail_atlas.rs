use xrf_dds::{DdsFile, RgbaImage};
use xrf_error::{XrfError, XrfResult};

use crate::level::build::level_build_atlas_place::LevelBuildAtlasPlace;
use crate::level::build::level_build_details::LevelBuildDetails;
use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;

/// A level's detail atlas, `build_details.dds`: every planted detail object's texture merged by
/// `CImageManager::CreateMergedTexture`, each whole at a pixel corner at its own size, turned or not.
pub(crate) struct LevelDetailAtlas {
  image: RgbaImage,
}

impl LevelDetailAtlas {
  /// How far a place's corner or size may stand from a whole pixel, past single-precision rounding.
  const PIXEL_TOLERANCE: f32 = 0.05;

  /// The atlas of the DDS file `build_details.dds`, read at its full size.
  ///
  /// # Errors
  ///
  /// Returns an error when the file is no DDS that decodes.
  pub(crate) fn read_from_bytes(bytes: Vec<u8>) -> XrfResult<Self> {
    Ok(Self {
      image: DdsFile::read_from_vec(bytes)?.decode_rgba(0)?,
    })
  }

  /// Texture `name` cut back out of the atlas at `place`, unturned, at the size it spans so a rebuild packs it as the
  /// level's was, and encoded as the atlas is ([`LevelBuildDetails::encode`]). It stands in for the library object's
  /// own texture.
  ///
  /// # Errors
  ///
  /// Returns an error when the place is not whole pixels inside the atlas, or the texture does not encode.
  pub(crate) fn cut(&self, name: &str, place: &LevelBuildAtlasPlace) -> XrfResult<LevelSceneTextureFile> {
    let (atlas_width, atlas_height) = self.image.dimensions();
    let (x, y) = Self::get_span(place.offset.x, place.scale.x, atlas_width)
      .zip(Self::get_span(place.offset.y, place.scale.y, atlas_height))
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Texture '{name}' stands at no whole pixels inside the {atlas_width} by {atlas_height} detail atlas"
        ))
      })?;
    let (width, height) = if place.is_rotated {
      (y.length, x.length)
    } else {
      (x.length, y.length)
    };
    let texture: RgbaImage = RgbaImage::from_fn(width, height, |texture_x, texture_y| {
      let (along_x, along_y) = if place.is_rotated {
        (texture_y, texture_x)
      } else {
        (texture_x, texture_y)
      };

      // Inside the atlas: each span ends within its side.
      *self.image.get_pixel(x.start + along_x, y.start + along_y)
    });

    Ok(LevelSceneTextureFile {
      name: String::from(name),
      bytes: LevelBuildDetails::encode(&texture)?,
      is_library: true,
    })
  }

  /// The pixels of a side `size` long covered from `offset` over `scale` of it, where both round to whole pixels and
  /// the span is non-empty and inside the side.
  fn get_span(offset: f32, scale: f32, size: u32) -> Option<LevelDetailAtlasSpan> {
    let start: u32 = Self::get_pixels(offset, size)?;
    let length: u32 = Self::get_pixels(scale, size)?;

    (length > 0 && u64::from(start) + u64::from(length) <= u64::from(size))
      .then_some(LevelDetailAtlasSpan { start, length })
  }

  /// `fraction` of `size` as a whole number of pixels, where it stands within rounding of one.
  fn get_pixels(fraction: f32, size: u32) -> Option<u32> {
    let pixels: f32 = fraction * size as f32;
    let whole: f32 = pixels.round();

    (whole >= 0.0 && (pixels - whole).abs() <= Self::PIXEL_TOLERANCE).then_some(whole as u32)
  }
}

/// The pixels a place covers along one side of the atlas.
#[derive(Clone, Copy)]
struct LevelDetailAtlasSpan {
  start: u32,
  length: u32,
}
