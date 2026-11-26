use xrf_dds::{D3DFormat, DdsFormat, DdsMetadata, DxgiFormat};
use xrf_error::{XrfError, XrfResult};
use xrf_thm::{ThmFile, ThmFormat, ThmTextureFlag, ThmTextureParamChunk, ThmTextureType};

/// The descriptor the SDK gives a texture that has none (`CImageManager::CreateTextureThumbnail`,
/// `xray/trunk/editors/ECore/Editor/ImageManager.cpp`), its format read from the encoded file rather than guessed from
/// whether it has alpha.
pub struct TextureEncodedDescriptor;

impl TextureEncodedDescriptor {
  /// The descriptor of the texture `metadata` describes, in `format` where the caller names one and otherwise in the
  /// one its layout is named by.
  ///
  /// A cube map records the strip of six faces the SDK built it from, and like the SDK it generates no mip maps of
  /// its own.
  ///
  /// # Errors
  ///
  /// Returns an error for a volume, which no descriptor describes, and, where the caller names no format, for a layout
  /// `ETFormat` has no member for, which a descriptor could only misname.
  pub fn of(metadata: &DdsMetadata, format: Option<ThmFormat>) -> XrfResult<ThmFile> {
    if metadata.is_volume {
      return Err(XrfError::new_texture_processing_error(
        "Cannot describe a volume texture, which no texture descriptor type names",
      ));
    }

    let format: ThmFormat = format.or_else(|| Self::format_of(metadata.format)).ok_or_else(|| {
      XrfError::new_texture_processing_error(format!(
        "Cannot describe a texture in {} without being told its format, which no descriptor format names",
        metadata.get_format_label()
      ))
    })?;
    let mut file: ThmFile = ThmFile::new_texture();
    let param: &mut ThmTextureParamChunk = file.texture_param.get_or_insert_default();

    param.format = format;
    param.width = metadata.width;
    param.height = metadata.height;
    param.flags.set(ThmTextureFlag::HasAlpha, format.has_alpha_channel());

    if metadata.is_cubemap {
      param.width = metadata.width * 6;
      param.flags.set(ThmTextureFlag::GenerateMipMaps, false);
      file.texture_type = Some(ThmTextureType::CubeMap);
    }

    Ok(file)
  }

  /// The `ETFormat` a DDS layout is named by, or `None` for one it has no member for.
  ///
  /// BC1 is always `tfDXT1`, never `tfADXT1`: the header cannot say whether its one bit of alpha is used.
  pub fn format_of(format: DdsFormat) -> Option<ThmFormat> {
    match format {
      DdsFormat::D3d(format) => match format {
        D3DFormat::DXT1 => Some(ThmFormat::Dxt1),
        D3DFormat::DXT3 => Some(ThmFormat::Dxt3),
        D3DFormat::DXT5 => Some(ThmFormat::Dxt5),
        D3DFormat::A8R8G8B8 | D3DFormat::A8B8G8R8 => Some(ThmFormat::Rgba),
        D3DFormat::R8G8B8 | D3DFormat::X8R8G8B8 | D3DFormat::X8B8G8R8 => Some(ThmFormat::Rgb),
        D3DFormat::R5G6B5 => Some(ThmFormat::Rgb565),
        D3DFormat::A1R5G5B5 | D3DFormat::X1R5G5B5 => Some(ThmFormat::Rgba1555),
        D3DFormat::A4R4G4B4 | D3DFormat::X4R4G4B4 => Some(ThmFormat::Rgba4444),
        D3DFormat::A8 => Some(ThmFormat::A8),
        D3DFormat::L8 => Some(ThmFormat::L8),
        D3DFormat::A8L8 => Some(ThmFormat::A8L8),
        _ => None,
      },
      DdsFormat::Dxgi(format) => match format {
        DxgiFormat::BC1_UNorm | DxgiFormat::BC1_UNorm_sRGB => Some(ThmFormat::Dxt1),
        DxgiFormat::BC2_UNorm | DxgiFormat::BC2_UNorm_sRGB => Some(ThmFormat::Dxt3),
        DxgiFormat::BC3_UNorm | DxgiFormat::BC3_UNorm_sRGB => Some(ThmFormat::Dxt5),
        DxgiFormat::R8G8B8A8_UNorm
        | DxgiFormat::R8G8B8A8_UNorm_sRGB
        | DxgiFormat::B8G8R8A8_UNorm
        | DxgiFormat::B8G8R8A8_UNorm_sRGB => Some(ThmFormat::Rgba),
        DxgiFormat::B8G8R8X8_UNorm | DxgiFormat::B8G8R8X8_UNorm_sRGB => Some(ThmFormat::Rgb),
        DxgiFormat::B5G6R5_UNorm => Some(ThmFormat::Rgb565),
        DxgiFormat::B5G5R5A1_UNorm => Some(ThmFormat::Rgba1555),
        DxgiFormat::B4G4R4A4_UNorm => Some(ThmFormat::Rgba4444),
        DxgiFormat::A8_UNorm => Some(ThmFormat::A8),
        _ => None,
      },
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use xrf_dds::{
    DdsEncoding, DdsFile, DdsMetadata, DdsMipChain, DdsMipFilter, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage,
  };
  use xrf_error::XrfResult;
  use xrf_test_utils::GeneratedResources;
  use xrf_thm::{ThmFile, ThmFormat, ThmTextureFlag, ThmTextureParamChunk, ThmTextureType};

  use super::TextureEncodedDescriptor;

  fn encoded(case: &str, format: ImageFormat, mipmaps: DdsMipmaps) -> XrfResult<DdsFile> {
    let path: PathBuf = GeneratedResources::path(&format!("xrf-texture/encoded_descriptor/{case}.dds"));

    std::fs::create_dir_all(path.parent().expect("case directory"))?;
    DdsEncoding::new(format, Quality::Fast)
      .encode(&DdsMipChain::build(
        RgbaImage::from_pixel(16, 8, Rgba([10, 20, 30, 40])),
        mipmaps,
      )?)?
      .write_to_path(&path)?;

    DdsFile::read_from_path(&path)
  }

  fn param(file: &ThmFile) -> ThmTextureParamChunk {
    file.texture_param.expect("a texture descriptor carries its parameters")
  }

  #[test]
  fn starts_from_the_sdk_defaults_with_the_size_and_format_of_the_file() -> XrfResult {
    let file: ThmFile = TextureEncodedDescriptor::of(
      &encoded(
        "bc3",
        ImageFormat::BC3RgbaUnorm,
        DdsMipmaps::Filtered(DdsMipFilter::Box),
      )?
      .metadata(),
      None,
    )?;
    let expected: ThmFile = ThmFile::new_texture();

    assert_eq!((param(&file).width, param(&file).height), (16, 8));
    assert_eq!(param(&file).format, ThmFormat::Dxt5);
    assert!(param(&file).flags.has(ThmTextureFlag::HasAlpha));
    assert!(param(&file).flags.has(ThmTextureFlag::GenerateMipMaps));
    assert_eq!(param(&file).mip_filter, param(&expected).mip_filter);
    assert_eq!(file.texture_type, Some(ThmTextureType::Image));
    assert_eq!(file.bump, expected.bump);
    assert_eq!(file.version, Some(ThmFile::VERSION));

    Ok(())
  }

  #[test]
  fn names_bc1_without_alpha_as_the_sdk_starts_it() -> XrfResult {
    let file: ThmFile = TextureEncodedDescriptor::of(
      &encoded("bc1", ImageFormat::BC1RgbaUnorm, DdsMipmaps::Disabled)?.metadata(),
      None,
    )?;

    assert_eq!(param(&file).format, ThmFormat::Dxt1);
    assert!(!param(&file).flags.has(ThmTextureFlag::HasAlpha));

    Ok(())
  }

  #[test]
  fn refuses_a_layout_no_descriptor_format_names_unless_told_one() -> XrfResult {
    let metadata: DdsMetadata = encoded("bc7", ImageFormat::BC7RgbaUnorm, DdsMipmaps::Disabled)?.metadata();

    assert!(TextureEncodedDescriptor::of(&metadata, None).is_err());
    assert_eq!(
      param(&TextureEncodedDescriptor::of(&metadata, Some(ThmFormat::Dxt5))?).format,
      ThmFormat::Dxt5
    );

    Ok(())
  }
}
