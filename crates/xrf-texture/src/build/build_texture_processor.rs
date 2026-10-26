use image::RgbaImage;
use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain};
use xrf_error::{XrfError, XrfResult};
use xrf_job::JobHandle;

use crate::build::build_texture_options::BuildTextureOptions;
use crate::build::build_texture_result::BuildTextureResult;
use crate::build::recipe::build_texture_bump_pair::BuildTextureBumpPair;
use crate::build::recipe::build_texture_layout::BuildTextureLayout;
use crate::build::recipe::build_texture_recipe::BuildTextureRecipe;
use crate::bump::{GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult};

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

    if let BuildTextureLayout::BumpPair(pair) = &recipe.layout {
      return Self::build_bump_pair(options, pair);
    }

    if options.normal_map.is_some() {
      return Err(XrfError::new_texture_processing_error(
        "Only a bump descriptor takes an external normal map, and this build was given one for another",
      ));
    }

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
      BuildTextureLayout::CubeStrip => encoding.encode_cube(&faces)?,
      BuildTextureLayout::Planar | BuildTextureLayout::BumpPair(_) => encoding.encode(&faces[0])?,
    };

    encoded.write_to_path(&options.destination)?;

    Ok(BuildTextureResult {
      destination: options.destination.clone(),
      width: source.width(),
      height: source.height(),
      mipmap_levels: levels,
      omissions: recipe.omissions,
      bump: None,
    })
  }

  /// The pair a bump descriptor builds, `DXTCompressBump`: the gloss read from the source's own alpha, the normals from
  /// its colour or from the external map the descriptor names, and both halves DXT5 with a box chain.
  ///
  /// # Errors
  ///
  /// Returns an error when the descriptor names an external normal map the build was not given, or was given one it
  /// does not name, when that map is one [`crate::BuildTextureNormalMap::accept`] refuses, or when either half cannot be
  /// written.
  fn build_bump_pair(options: &BuildTextureOptions, pair: &BuildTextureBumpPair) -> XrfResult<BuildTextureResult> {
    let source: &RgbaImage = &options.source;
    let normal_map: Option<RgbaImage> = match (&pair.normal_map, &options.normal_map) {
      (Some(name), Some(normal_map)) => Some(normal_map.accept(name, source.dimensions())?.clone()),
      (Some(name), None) => {
        return Err(XrfError::new_texture_processing_error(format!(
          "The bump descriptor names the external normal map '{name}', which this build was not given"
        )));
      }
      (None, Some(_)) => {
        return Err(XrfError::new_texture_processing_error(
          "This build was given an external normal map the bump descriptor does not name",
        ));
      }
      (None, None) => None,
    };
    let generated: GenerateBumpResult = GenerateBumpProcessor::generate(&GenerateBumpOptions {
      job: JobHandle::inert(),
      bump: BuildTextureBumpPair::bump_of(&options.destination),
      height: source.clone(),
      gloss: GenerateBumpGloss::HeightAlpha,
      normal_map,
      virtual_height: pair.virtual_height,
      mip_filter: BuildTextureBumpPair::MIP_FILTER,
      quality: options.quality,
    })?;

    Ok(BuildTextureResult {
      destination: generated.bump.clone(),
      width: source.width(),
      height: source.height(),
      mipmap_levels: generated.mipmap_levels,
      omissions: Vec::new(),
      bump: Some(generated),
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
  use crate::build::build_texture_normal_map::BuildTextureNormalMap;
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
      normal_map: None,
      quality: Quality::Fast,
    }
  }

  /// A bump descriptor, naming the external normal map `normal_map` when given one.
  fn bump_descriptor(normal_map: Option<&str>) -> ThmFile {
    let mut bump: ThmFile = descriptor(ThmFormat::A8, &[], ThmMipFilter::Advanced);

    bump.texture_type = Some(ThmTextureType::BumpMap);
    bump.ext_normal_map_name = normal_map.map(str::to_owned);
    bump
  }

  /// A normal map as the SDK takes one: a normal-map descriptor of 32-bit RGBA, and its pixels.
  fn normal_map(size: u32) -> BuildTextureNormalMap {
    let mut descriptor: ThmFile = descriptor(ThmFormat::Rgba, &[], ThmMipFilter::Box);

    descriptor.texture_type = Some(ThmTextureType::NormalMap);

    BuildTextureNormalMap {
      descriptor,
      image: RgbaImage::from_pixel(size, size, Rgba([128, 128, 255, 255])),
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
  fn builds_a_bump_descriptor_as_its_pair_with_the_gloss_from_the_source_alpha() -> XrfResult {
    // The descriptor's own format and filter are ones a plain build refuses or reduces itself; `DXTCompressBump` reads
    // neither, so the pair is built anyway, DXT5 with a box chain, and leaves nothing out.
    let source: RgbaImage = RgbaImage::from_pixel(16, 16, Rgba([128, 128, 128, 0x40]));
    let built: BuildTextureResult =
      BuildTextureProcessor::build(&options("stone_bump", bump_descriptor(None), source))?;
    let pair = built.bump.as_ref().expect("a bump descriptor builds a pair");

    assert_eq!(built.destination.file_name().unwrap(), "stone_bump.dds");
    assert_eq!(pair.companion.file_name().unwrap(), "stone_bump#.dds");
    assert!(pair.companion.is_file());
    assert!(
      (pair.gloss_power - 64.0 / 255.0).abs() < 1e-6,
      "got {}",
      pair.gloss_power
    );
    assert!(built.omissions.is_empty());
    assert_eq!(built.mipmap_levels, 5);

    Ok(())
  }

  #[test]
  fn takes_the_external_normal_map_a_bump_descriptor_names_as_the_sdk_does() -> XrfResult {
    let named: ThmFile = bump_descriptor(Some("stone\\stone_nmap"));
    let mut given: BuildTextureOptions = options("named_bump", named.clone(), translucent(16));

    given.normal_map = Some(normal_map(16));

    assert!(BuildTextureProcessor::build(&given)?.bump.is_some());

    // Named and not given, given and not named, or given in a shape the SDK refuses: each fails rather than building
    // a pair from normals the descriptor did not ask for.
    assert!(BuildTextureProcessor::build(&options("named_missing", named.clone(), translucent(16))).is_err());

    let mut unnamed: BuildTextureOptions = options("unnamed_bump", bump_descriptor(None), translucent(16));

    unnamed.normal_map = Some(normal_map(16));

    assert!(BuildTextureProcessor::build(&unnamed).is_err());

    let mut mismatched: BuildTextureOptions = options("mismatched_bump", named, translucent(16));

    mismatched.normal_map = Some(normal_map(8));

    assert!(BuildTextureProcessor::build(&mismatched).is_err());

    let mut image: BuildTextureOptions = options(
      "image_with_normals",
      descriptor(ThmFormat::Dxt5, &[], ThmMipFilter::Box),
      translucent(16),
    );

    image.normal_map = Some(normal_map(16));

    assert!(BuildTextureProcessor::build(&image).is_err());

    Ok(())
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
