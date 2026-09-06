use std::path::{Path, PathBuf};

use image::RgbaImage;
use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain, DdsMipmaps, ImageFormat};
use xrf_error::{XrfError, XrfResult};

use crate::bump::bump_normal_map::{GreyPlane, derive_normal_map};
use crate::bump::generate_bump_options::{GenerateBumpGloss, GenerateBumpOptions};
use crate::bump::generate_bump_result::GenerateBumpResult;
use crate::image_file::DDS_EXTENSION;

/// Suffix of the half holding the normals and the gloss.
const BUMP_SUFFIX: &str = "_bump";

/// Suffix of the half holding the compression error and the height.
const COMPANION_SUFFIX: &str = "_bump#";

/// The value a difference of nothing encodes to, so an error plane is signed around the middle of the byte range.
const ERROR_BIAS: i32 = 128;

/// The middle the height plane is shifted onto, one below the midpoint the SDK uses everywhere else.
const HEIGHT_CENTER: i32 = 127;

/// Builds the two textures the renderer binds for a bumped surface.
///
/// A port of `DXTCompressBump` (`xray/trunk/xrDXT/NormalMapGen.cpp`), which is the only description of this format
/// that exists: the pair is not two pictures but one packed structure, and the second half is defined in terms of what
/// the first half lost to compression. That is why the encoder runs in the middle of the algorithm rather than at the
/// end of it - the error plane is the difference between the normals and the normals as DXT5 gave them back, so it
/// can only be computed against the encoder that will actually write the file.
pub struct GenerateBumpProcessor {}

impl GenerateBumpProcessor {
  /// Generates the pair and writes both halves beside each other.
  ///
  /// # Errors
  ///
  /// Returns an error when an external normal map does not match the height in size, when either half cannot be
  /// encoded, or when either file cannot be written. A gloss too dark to be useful is reported in the result rather
  /// than refused; see [`GenerateBumpResult::is_gloss_too_dark`].
  pub fn generate(options: &GenerateBumpOptions) -> XrfResult<GenerateBumpResult> {
    let height: GreyPlane = GreyPlane::from_rgb_average(&options.height);
    let normals: RgbaImage = Self::resolve_normals(options, &height)?;
    let gloss: GreyPlane = Self::resolve_gloss(options)?;

    // The first half as the engine reads it: gloss in red, and the normal's three channels reversed behind it.
    let bump: RgbaImage = Self::pack_bump(&normals, &gloss);
    let bump_path: PathBuf = Self::half_path(&options.destination, BUMP_SUFFIX);

    Self::encode(&bump, options)?.write_to_path(&bump_path)?;

    // Read back rather than kept, because what the second half corrects is what the file has, not what was handed to
    // the encoder.
    let restored: RgbaImage = DdsFile::read_from_path(&bump_path)?.decode_rgba(0)?;
    let companion: RgbaImage = Self::pack_companion(&bump, &restored, &Self::center_height(options, &height));
    let companion_path: PathBuf = Self::half_path(&options.destination, COMPANION_SUFFIX);

    Self::encode(&companion, options)?.write_to_path(&companion_path)?;

    Ok(GenerateBumpResult {
      bump: bump_path,
      companion: companion_path,
      gloss_power: gloss.average(),
    })
  }

  /// The normals the pair is built on: the ones a descriptor names, or the ones the height implies.
  ///
  /// An external map replaces the derivation outright rather than blending with it, which is what makes it worth
  /// naming: an author who has real normals is not asking for them to be inferred again.
  fn resolve_normals(options: &GenerateBumpOptions, height: &GreyPlane) -> XrfResult<RgbaImage> {
    let Some(normal_map) = &options.normal_map else {
      return Ok(derive_normal_map(height, options.virtual_height));
    };

    if normal_map.dimensions() != options.height.dimensions() {
      return Err(XrfError::new_texture_processing_error(format!(
        "External normal map is {}x{} and the height is {}x{}, which cannot be packed together",
        normal_map.width(),
        normal_map.height(),
        options.height.width(),
        options.height.height()
      )));
    }

    Ok(normal_map.clone())
  }

  /// The gloss plane, whether it came as a mask or as one value for the whole surface.
  fn resolve_gloss(options: &GenerateBumpOptions) -> XrfResult<GreyPlane> {
    match &options.gloss {
      GenerateBumpGloss::Mask(mask) => {
        if mask.dimensions() != options.height.dimensions() {
          return Err(XrfError::new_texture_processing_error(format!(
            "Gloss mask is {}x{} and the height is {}x{}, which cannot be packed together",
            mask.width(),
            mask.height(),
            options.height.width(),
            options.height.height()
          )));
        }

        Ok(GreyPlane::from_rgb_average(mask))
      }
      GenerateBumpGloss::Constant(level) => Ok(GreyPlane::from_rgb_average(&RgbaImage::from_pixel(
        options.height.width(),
        options.height.height(),
        image::Rgba([to_byte(*level * 255.0); 4]),
      ))),
    }
  }

  /// The first half: gloss in red, the normal reversed into the other three (`it_gloss_rev`).
  ///
  /// The reversal is what lets the shader read `Nu.wzy` as the normal, and the gloss is stored one above what it was
  /// given, which the SDK does without explanation and which matters at the dark end.
  fn pack_bump(normals: &RgbaImage, gloss: &GreyPlane) -> RgbaImage {
    RgbaImage::from_fn(normals.width(), normals.height(), |x, y| {
      let normal = normals.get_pixel(x, y).0;
      let level: u8 = to_byte(gloss.wrapped(i64::from(x), i64::from(y)) * 255.0);

      image::Rgba([level.saturating_add(1), normal[2], normal[1], normal[0]])
    })
  }

  /// The second half: what the first half lost, and the height (`it_difference` then `it_height_rev`).
  ///
  /// The error is two thirds of the difference biased onto the middle of the byte range, which is what the shader
  /// undoes when it adds `NuE.xyz - 1`. The height goes into alpha, where the parallax path reads it.
  fn pack_companion(bump: &RgbaImage, restored: &RgbaImage, height: &[u8]) -> RgbaImage {
    RgbaImage::from_fn(bump.width(), bump.height(), |x, y| {
      let original = bump.get_pixel(x, y).0;
      let decoded = restored.get_pixel(x, y).0;
      let error = |channel: usize| {
        to_byte_from_int(ERROR_BIAS + 2 * (i32::from(original[channel]) - i32::from(decoded[channel])) / 3)
      };

      // Reversed once more, so the error lines up channel for channel with the normal the shader rebuilds.
      image::Rgba([error(3), error(2), error(1), height[(y * bump.width() + x) as usize]])
    })
  }

  /// The height plane as it is stored: rescaled by the virtual height, then shifted so its middle sits at the centre.
  ///
  /// Centring is what makes parallax behave the same on a surface authored bright as on one authored dark. The middle
  /// is nine parts the mean and one part the midpoint between the extremes, so one very bright or very dark region
  /// moves it a little and a generally bright surface moves it a lot.
  fn center_height(options: &GenerateBumpOptions, height: &GreyPlane) -> Vec<u8> {
    let scale: f32 = (options.virtual_height / GenerateBumpOptions::DEFAULT_VIRTUAL_HEIGHT).powf(0.75);
    // Past one the rescale is softened, so a tall surface does not flatten against the top of the byte range.
    let scale: f32 = if scale > 1.0 { scale.sqrt() } else { scale };

    let scaled: Vec<u8> = (0..height.height)
      .flat_map(|y| {
        (0..height.width).map(move |x| to_byte((height.wrapped(i64::from(x), i64::from(y)) * 255.0 * scale).floor()))
      })
      .collect();

    let total: u32 = scaled.iter().map(|level| u32::from(*level)).sum();
    let average: u32 = total / scaled.len() as u32;
    let lowest: u32 = u32::from(*scaled.iter().min().unwrap_or(&0));
    let highest: u32 = u32::from(*scaled.iter().max().unwrap_or(&0));
    let median: u32 = (9 * average + (highest - lowest) / 2 + lowest) / 10;
    let correction: i32 = HEIGHT_CENTER - median as i32;

    scaled
      .into_iter()
      .map(|level| to_byte_from_int(i32::from(level) + correction))
      .collect()
  }

  /// Both halves are written the same way: DXT5, with a mip chain, as `fmt0` in the SDK sets up twice.
  fn encode(image: &RgbaImage, options: &GenerateBumpOptions) -> XrfResult<DdsFile> {
    DdsEncoding::new(ImageFormat::BC3RgbaUnorm, options.quality)
      .encode(&DdsMipChain::build(image, DdsMipmaps::Filtered(options.mip_filter))?)
  }

  /// Where one half of the pair is written, from the base name both share.
  fn half_path(destination: &Path, suffix: &str) -> PathBuf {
    let name: String = destination
      .file_name()
      .map_or_else(String::new, |name| name.to_string_lossy().into_owned());

    destination.with_file_name(format!("{name}{suffix}.{DDS_EXTENSION}"))
  }
}

/// Rounds a computed level into a byte, clamping rather than wrapping at either end.
fn to_byte(value: f32) -> u8 {
  value.clamp(0.0, 255.0) as u8
}

/// The same clamp for a value that is already whole, which is where the error and the centring land.
fn to_byte_from_int(value: i32) -> u8 {
  value.clamp(0, 255) as u8
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use image::{Rgba, RgbaImage};
  use xrf_dds::{DdsFile, DdsMipFilter, Quality};
  use xrf_error::XrfResult;
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::GenerateBumpProcessor;
  use crate::bump::generate_bump_options::{GenerateBumpGloss, GenerateBumpOptions};
  use crate::bump::generate_bump_result::GenerateBumpResult;

  /// A height map of one level everywhere, written into all three colour channels.
  fn level(size: u32, height: u8) -> RgbaImage {
    RgbaImage::from_pixel(size, size, Rgba([height, height, height, u8::MAX]))
  }

  fn options(case: &str, height: RgbaImage) -> GenerateBumpOptions {
    let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("xrf-texture/bump/{case}"));

    std::fs::create_dir_all(destination.parent().expect("case directory")).expect("scratch directory");

    GenerateBumpOptions {
      destination,
      height,
      gloss: GenerateBumpGloss::Constant(0.5),
      normal_map: None,
      virtual_height: GenerateBumpOptions::DEFAULT_VIRTUAL_HEIGHT,
      mip_filter: DdsMipFilter::Box,
      quality: Quality::Fast,
    }
  }

  /// The base level of a written half, as the four channels it stores.
  fn decode(path: &PathBuf) -> XrfResult<RgbaImage> {
    DdsFile::read_from_path(path)?.decode_rgba(0)
  }

  #[test]
  fn a_flat_surface_packs_to_the_values_the_shader_reads_as_up() -> XrfResult {
    // Every block is one colour, so DXT5 reproduces it exactly and the bytes below are arithmetic rather than a
    // tolerance. A flat height derives the normal `(0, 0, 1)`, which packs to `128, 128, 255`.
    let result: GenerateBumpResult = GenerateBumpProcessor::generate(&options("flat", level(8, 128)))?;

    // Gloss in red, then the normal reversed: `x` into alpha, `y` into blue, `z` into green. Gloss is stored one
    // above the 127 that `0.5` rounds down to, which is the `+1` the SDK applies without explanation.
    let bump: [u8; 4] = decode(&result.bump)?.get_pixel(0, 0).0;

    // Alpha is exact, and that is the point of the whole reversal: DXT5 keeps alpha at eight bits and squeezes
    // colour into 5:6:5, so the format puts the value that must be precise - the normal's `x` - where precision
    // survives. 127 is the flat normal the shipped textures use.
    assert_eq!(bump[3], 127);

    // The other three are what 5:6:5 makes of `128, 255, 127`: red and blue lose three bits each and come back four
    // off, green loses two and survives at the top of its range.
    assert_eq!([bump[0], bump[1], bump[2]], [132, 255, 123]);

    // Nothing was lost, so every error channel is the bias itself. Height is centred: the plane is 128 everywhere, so
    // its median is 128 and the correction moves it one down to the 127 the format centres on.
    let companion: [u8; 4] = decode(&result.companion)?.get_pixel(0, 0).0;

    // Height is the second value that must be precise, and it is in alpha for the same reason. The plane is 128
    // everywhere, so its median is 128 and the correction moves it one down onto the 127 the format centres on.
    assert_eq!(companion[3], 127);

    // Nothing was lost encoding a flat normal beyond the 5:6:5 quantisation itself, so every error channel is the
    // bias, 128, as 5:6:5 returns it.
    assert_eq!([companion[0], companion[1], companion[2]], [132, 130, 132]);

    // What the two make when the shader adds them: `Nu.wzy + (NuE.xyz - 1)` is
    // `(127, 123, 255) / 255 + (132, 130, 132) / 255 - 1`, which is `(0.02, -0.008, 0.518)` - straight up at the half
    // scale the packing stores, with the error plane having pulled blue's quantisation most of the way back.
    Ok(())
  }

  #[test]
  fn the_companion_carries_the_height_in_alpha_where_parallax_reads_it() -> XrfResult {
    // Two bands, each a whole number of 4x4 blocks, so the alpha of each block is uniform and survives DXT5 exactly.
    let mut height: RgbaImage = level(8, 64);

    for y in 4..8 {
      for x in 0..8 {
        height.get_pixel_mut(x, y).0 = [192, 192, 192, u8::MAX];
      }
    }

    let result: GenerateBumpResult = GenerateBumpProcessor::generate(&options("bands", height))?;
    let companion: RgbaImage = decode(&result.companion)?;

    // Centred on the median, which for two equal bands is the mean, 128: the correction is `127 - 128`, so the bands
    // land one below where they started.
    assert_eq!(companion.get_pixel(0, 0).0[3], 63);
    assert_eq!(companion.get_pixel(0, 7).0[3], 191);

    Ok(())
  }

  #[test]
  fn a_taller_virtual_height_stretches_the_stored_relief() -> XrfResult {
    // The rescale is `(virtual / 0.05) ^ 0.75`, softened by a square root past one, and it is the only thing that
    // changes the distance between two heights.
    let mut height: RgbaImage = level(8, 100);

    for y in 4..8 {
      for x in 0..8 {
        height.get_pixel_mut(x, y).0 = [140, 140, 140, u8::MAX];
      }
    }

    let mut taller: GenerateBumpOptions = options("taller", height.clone());

    taller.virtual_height = 0.2;

    let spread = |result: &GenerateBumpResult| -> XrfResult<u8> {
      let companion: RgbaImage = decode(&result.companion)?;

      Ok(companion.get_pixel(0, 7).0[3] - companion.get_pixel(0, 0).0[3])
    };

    let gentle: u8 = spread(&GenerateBumpProcessor::generate(&options("gentle", height))?)?;
    let stretched: u8 = spread(&GenerateBumpProcessor::generate(&taller)?)?;

    // Four times the height is `4 ^ 0.75` softened by a square root, about 1.68, and the two bands 40 apart come out
    // around 67 apart.
    assert!(
      stretched >= gentle * 3 / 2,
      "expected {gentle} to stretch by half again, got {stretched}"
    );

    Ok(())
  }

  #[test]
  fn the_gloss_power_is_the_mean_of_the_surface_and_names_a_dark_one() -> XrfResult {
    let bright: GenerateBumpResult = GenerateBumpProcessor::generate(&options("bright", level(8, 128)))?;

    // The slider's `0.5` reaches the plane as the byte 127, so the mean it reports is `127 / 255`.
    assert!(
      (bright.gloss_power - 127.0 / 255.0).abs() < 1e-3,
      "got {}",
      bright.gloss_power
    );
    assert!(!bright.is_gloss_too_dark());

    let mut dark: GenerateBumpOptions = options("dark", level(8, 128));

    dark.gloss = GenerateBumpGloss::Constant(0.05);

    let dark: GenerateBumpResult = GenerateBumpProcessor::generate(&dark)?;

    // A verdict, not a failure: both halves are still on disk.
    assert!(dark.is_gloss_too_dark());
    assert!(dark.bump.is_file() && dark.companion.is_file());

    Ok(())
  }

  #[test]
  fn the_two_halves_are_named_after_the_texture_they_belong_to() -> XrfResult {
    let result: GenerateBumpResult = GenerateBumpProcessor::generate(&options("naming", level(8, 128)))?;

    assert_eq!(result.bump.file_name().unwrap(), "naming_bump.dds");
    assert_eq!(result.companion.file_name().unwrap(), "naming_bump#.dds");

    Ok(())
  }

  #[test]
  fn refuses_a_normal_map_or_gloss_mask_that_does_not_match_the_height() {
    let mut mismatched: GenerateBumpOptions = options("mismatch", level(8, 128));

    mismatched.normal_map = Some(level(4, 128));

    assert!(GenerateBumpProcessor::generate(&mismatched).is_err());

    let mut mismatched: GenerateBumpOptions = options("mismatch_gloss", level(8, 128));

    mismatched.gloss = GenerateBumpGloss::Mask(level(4, 200));

    assert!(GenerateBumpProcessor::generate(&mismatched).is_err());
  }
}
