use image::RgbaImage;
use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain};
use xrf_error::{XrfError, XrfResult};

use crate::build::build_texture_options::BuildTextureOptions;
use crate::build::build_texture_result::BuildTextureResult;
use crate::build::recipe::build_texture_layout::BuildTextureLayout;
use crate::build::recipe::build_texture_recipe::BuildTextureRecipe;

/// Rebuilds a texture from a source image, the way its descriptor says to.
///
/// The descriptor decides everything: whether the source is one face or a cube's six, the format it is written in,
/// whether there is a mip chain and who reduces it, and whether the source's alpha means anything. Nothing is taken
/// from the file being replaced, so a rebuild is reproducible from the two inputs alone.
pub struct BuildTextureProcessor {}

impl BuildTextureProcessor {
  /// Builds the texture and writes it, answering what it did and what it left out.
  ///
  /// # Errors
  ///
  /// Returns an error when the descriptor is one [`BuildTextureRecipe::of`] refuses, when the source has no pixels or
  /// is not the shape its layout and mip chain need, or when the encoded texture cannot be written. A recipe field that
  /// is merely not implemented is reported in the result instead; see [`crate::BuildTextureOmission`].
  pub fn build(options: &BuildTextureOptions) -> XrfResult<BuildTextureResult> {
    let recipe: BuildTextureRecipe = BuildTextureRecipe::of(&options.descriptor)?;
    let source: &RgbaImage = &options.source;
    let faces: Vec<DdsMipChain> = recipe
      .layout
      .split(source)?
      .iter()
      .map(|face| recipe.mipmaps.build(face))
      .collect::<XrfResult<_>>()?;
    let levels: u32 = u32::try_from(faces[0].levels().len())
      .map_err(|_| XrfError::new_texture_processing_error("Built texture has more levels than the format can carry"))?;
    let encoding: DdsEncoding = DdsEncoding::new(recipe.format, options.quality);
    let encoded: DdsFile = match recipe.layout {
      BuildTextureLayout::Planar => encoding.encode(&faces[0])?,
      BuildTextureLayout::CubeStrip => encoding.encode_cube(&faces)?,
    };

    encoded.write_to_path(&options.destination)?;

    Ok(BuildTextureResult {
      destination: options.destination.clone(),
      width: source.width(),
      height: source.height(),
      mipmap_levels: levels,
      omissions: recipe.omissions,
    })
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use image::{Rgba, RgbaImage};
  use xrf_dds::{DdsFile, Quality};
  use xrf_error::XrfResult;
  use xrf_test_utils::GeneratedResources;
  use xrf_thm::{
    ThmFile, ThmFormat, ThmMipFilter, ThmTextureFlag, ThmTextureFlags, ThmTextureParamChunk, ThmTextureType,
  };

  use super::BuildTextureProcessor;
  use crate::build::build_texture_options::BuildTextureOptions;
  use crate::build::build_texture_result::BuildTextureResult;
  use crate::build::recipe::build_texture_omission::BuildTextureOmission;

  /// A descriptor of the given format, with whichever flags a case is about.
  fn descriptor(format: ThmFormat, flags: &[ThmTextureFlag], mip_filter: ThmMipFilter) -> ThmFile {
    let mut file: ThmFile = ThmFile::new_texture();
    // Started from an empty flag word rather than the SDK's defaults, so each case's list is exhaustive: the
    // defaults set mip map generation and colour dithering, which are two of the things these cases are about.
    let mut param: ThmTextureParamChunk = ThmTextureParamChunk {
      format,
      mip_filter,
      flags: ThmTextureFlags::none(),
      ..ThmTextureParamChunk::default()
    };

    for flag in flags {
      param.flags.set(*flag, true);
    }

    file.texture_param = Some(param);
    file
  }

  fn options(case: &str, descriptor: ThmFile, source: RgbaImage) -> BuildTextureOptions {
    let destination: PathBuf = GeneratedResources::path(&format!("xrf-texture/build/{case}.dds"));

    std::fs::create_dir_all(destination.parent().expect("case directory")).expect("scratch directory");

    BuildTextureOptions {
      destination,
      source,
      descriptor,
      quality: Quality::Fast,
    }
  }

  /// A picture with a half-transparent corner, so discarding alpha is visible.
  fn translucent(size: u32) -> RgbaImage {
    RgbaImage::from_fn(size, size, |x, _| {
      Rgba([200, 100, 50, if x < size / 2 { 0 } else { u8::MAX }])
    })
  }

  /// Six faces of [`translucent`] side by side, a cube's source.
  fn translucent_strip(size: u32) -> RgbaImage {
    let face: RgbaImage = translucent(size);

    RgbaImage::from_fn(size * 6, size, |x, y| *face.get_pixel(x % size, y))
  }

  #[test]
  fn writes_the_layout_and_the_chain_the_descriptor_asks_for() -> XrfResult {
    let built: BuildTextureResult = BuildTextureProcessor::build(&options(
      "dxt5_chain",
      descriptor(
        ThmFormat::Dxt5,
        &[ThmTextureFlag::GenerateMipMaps],
        ThmMipFilter::Kaiser,
      ),
      translucent(16),
    ))?;

    assert_eq!((built.width, built.height), (16, 16));
    assert_eq!(built.mipmap_levels, 5);
    assert!(built.omissions.is_empty(), "got {:?}", built.omissions);

    let written = DdsFile::read_from_path(&built.destination)?.metadata();

    assert_eq!(written.mipmap_levels, 5);
    assert_eq!((written.width, written.height), (16, 16));

    Ok(())
  }

  #[test]
  fn a_descriptor_without_mipmaps_writes_only_the_base() -> XrfResult {
    let built: BuildTextureResult = BuildTextureProcessor::build(&options(
      "flat",
      descriptor(ThmFormat::Dxt5, &[], ThmMipFilter::Box),
      translucent(16),
    ))?;

    assert_eq!(built.mipmap_levels, 1);

    Ok(())
  }

  #[test]
  fn dxt1_comes_back_opaque_and_its_alpha_bearing_twin_is_refused() -> XrfResult {
    // `tfDXT1` says the alpha means nothing, and BC1 here writes none, so the two agree and the build is faithful.
    let opaque: BuildTextureResult = BuildTextureProcessor::build(&options(
      "dxt1",
      descriptor(ThmFormat::Dxt1, &[], ThmMipFilter::Box),
      translucent(16),
    ))?;

    assert_eq!(
      DdsFile::read_from_path(&opaque.destination)?
        .decode_rgba(0)?
        .get_pixel(0, 0)
        .0[3],
      u8::MAX
    );

    // `tfADXT1` says the opposite, and the same encoder cannot express it. Refused, because writing it would turn
    // every transparent texel of a fence or a leaf solid - a visible change, not a recipe field left out.
    assert!(
      BuildTextureProcessor::build(&options(
        "dxt1a",
        descriptor(ThmFormat::Dxt1Alpha, &[], ThmMipFilter::Box),
        translucent(16),
      ))
      .is_err()
    );

    Ok(())
  }

  #[test]
  fn names_every_recipe_field_it_does_not_carry_out() -> XrfResult {
    // A fade over a chain nvDXT reduces is nvDXT's own, so it is named; the dithering beside it is not, because nvDXT
    // never dithered the formats written here.
    let built: BuildTextureResult = BuildTextureProcessor::build(&options(
      "omissions",
      descriptor(
        ThmFormat::Dxt5,
        &[
          ThmTextureFlag::GenerateMipMaps,
          ThmTextureFlag::DitherColor,
          ThmTextureFlag::FadeToColor,
          ThmTextureFlag::ColorBorder,
        ],
        ThmMipFilter::Kaiser,
      ),
      translucent(16),
    ))?;

    assert_eq!(
      built.omissions,
      vec![BuildTextureOmission::ColorBorder, BuildTextureOmission::FadeToColor]
    );

    Ok(())
  }

  #[test]
  fn the_advanced_filter_fades_as_the_sdk_does_and_leaves_nothing_out() -> XrfResult {
    // The SDK reduces this chain itself, fade included, so the same flags leave nothing to report.
    let built: BuildTextureResult = BuildTextureProcessor::build(&options(
      "advanced",
      descriptor(
        ThmFormat::Dxt5,
        &[
          ThmTextureFlag::GenerateMipMaps,
          ThmTextureFlag::DitherColor,
          ThmTextureFlag::FadeToColor,
        ],
        ThmMipFilter::Advanced,
      ),
      translucent(16),
    ))?;

    assert!(built.omissions.is_empty(), "got {:?}", built.omissions);
    assert_eq!(built.mipmap_levels, 5);

    Ok(())
  }

  #[test]
  fn writes_a_cube_from_six_faces_side_by_side() -> XrfResult {
    let mut cube: ThmFile = descriptor(ThmFormat::Rgba, &[], ThmMipFilter::Box);

    cube.texture_type = Some(ThmTextureType::CubeMap);

    let built: BuildTextureResult = BuildTextureProcessor::build(&options("cube", cube.clone(), translucent_strip(4)))?;
    let written = DdsFile::read_from_path(&built.destination)?.metadata();

    assert!(written.is_cubemap);
    assert_eq!((written.width, written.height), (4, 4));
    assert_eq!(written.data_size, 6 * 4 * 4 * 4, "six faces of sixteen RGBA texels");
    assert_eq!(
      (built.width, built.height),
      (24, 4),
      "the descriptor records the strip it was built from"
    );
    assert!(BuildTextureProcessor::build(&options("not_a_cube", cube, translucent(16))).is_err());

    Ok(())
  }

  #[test]
  fn refuses_a_bump_descriptor_which_builds_a_pair_rather_than_one_texture() {
    let mut bump: ThmFile = descriptor(ThmFormat::Dxt5, &[], ThmMipFilter::Box);

    bump.texture_type = Some(ThmTextureType::BumpMap);

    assert!(BuildTextureProcessor::build(&options("bump", bump, translucent(16))).is_err());
  }

  #[test]
  fn refuses_a_format_it_cannot_write_rather_than_substituting_one() {
    // 271 corpus descriptors name `tfA8`. Writing BC3 instead would load and would not be the texture described.
    let refused = BuildTextureProcessor::build(&options(
      "unsupported",
      descriptor(ThmFormat::A8, &[], ThmMipFilter::Box),
      translucent(16),
    ));

    assert!(refused.is_err());
  }
}
