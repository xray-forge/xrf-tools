use std::fs::File;
use std::io::{BufWriter, Cursor, Read, Write};
use std::path::Path;

use ddsfile::{Dds, DxgiFormat};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, RgbaImage};
use image_dds::{ImageFormat, Mipmaps, Quality, dds_from_image};
use xrf_error::{XrfError, XrfResult};

use crate::uncompressed::decode_uncompressed;
use crate::{DdsMetadata, DdsPng};

/// Bytes a DDS header occupies, with and without the DX10 extension.
const HEADER_SIZE: u64 = 128;
const DX10_HEADER_SIZE: u64 = 148;
const MAXIMUM_HEADER_SIZE: u64 = DX10_HEADER_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DdsEncodeOptions {
  pub format: ImageFormat,
  pub quality: Quality,
  pub mipmaps: Mipmaps,
}

impl DdsEncodeOptions {
  pub fn new(format: ImageFormat, quality: Quality, mipmaps: Mipmaps) -> Self {
    Self {
      format,
      quality,
      mipmaps,
    }
  }
}

/// Parsed DDS file with format behavior behind one interface.
pub struct DdsFile {
  dds: Dds,
  file_size: u64,
  metadata_size: u64,
}

impl DdsFile {
  pub fn read_from_path<P: AsRef<Path>>(path: P) -> XrfResult<Self> {
    let mut file: File = File::open(path.as_ref())?;
    let file_size: u64 = file.metadata()?.len();
    let dds: Dds = Dds::read(&mut file).map_err(|error| {
      XrfError::new_texture_processing_error(format!(
        "Failed to read texture by path {}, error: {}",
        path.as_ref().display(),
        error,
      ))
    })?;

    Self::from_parsed(dds, file_size)
  }

  /// Read a texture's header without its payload.
  ///
  /// A DDS header is at most 148 bytes, and `Dds::read` reads the payload with `read_to_end` and validates no length
  /// against it, so a reader limited to the header yields a complete header and an empty payload. That matters at scale:
  /// surveying the reference trees means answering format questions about ten gigabytes of textures, and the answers all
  /// live in the first 148 bytes of each file.
  ///
  /// The payload size is derived from the file length rather than measured, since the payload was never read.
  pub fn read_metadata_from_path<P: AsRef<Path>>(path: P) -> XrfResult<DdsMetadata> {
    let file: File = File::open(path.as_ref())?;
    let file_size: u64 = file.metadata()?.len();
    let dds: Dds = Dds::read(&mut file.take(MAXIMUM_HEADER_SIZE)).map_err(|error| {
      XrfError::new_texture_processing_error(format!(
        "Failed to read texture header by path {}, error: {}",
        path.as_ref().display(),
        error,
      ))
    })?;

    let metadata_size: u64 = if dds.header10.is_some() {
      DX10_HEADER_SIZE
    } else {
      HEADER_SIZE
    };

    Ok(DdsMetadata {
      data_size: usize::try_from(file_size.saturating_sub(metadata_size))
        .map_err(|_| XrfError::new_texture_processing_error("DDS payload exceeds the supported size range"))?,
      ..DdsMetadata::from_dds(&dds, file_size, metadata_size)
    })
  }

  /// Reads header facts from bytes already in hand, without keeping the payload.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes do not begin with a readable DDS header.
  pub fn read_metadata_from_bytes(bytes: &[u8]) -> XrfResult<DdsMetadata> {
    let file_size: u64 = u64::try_from(bytes.len())
      .map_err(|_| XrfError::new_texture_processing_error("DDS input exceeds the supported size range"))?;
    let header: &[u8] = &bytes[..bytes.len().min(MAXIMUM_HEADER_SIZE as usize)];
    let dds: Dds = Dds::read(&mut Cursor::new(header)).map_err(|error| {
      XrfError::new_texture_processing_error(format!("Failed to read DDS header from memory: {error}."))
    })?;

    let metadata_size: u64 = if dds.header10.is_some() {
      DX10_HEADER_SIZE
    } else {
      HEADER_SIZE
    };

    Ok(DdsMetadata {
      data_size: usize::try_from(file_size.saturating_sub(metadata_size))
        .map_err(|_| XrfError::new_texture_processing_error("DDS payload exceeds the supported size range"))?,
      ..DdsMetadata::from_dds(&dds, file_size, metadata_size)
    })
  }

  pub fn read_from_bytes(bytes: &[u8]) -> XrfResult<Self> {
    let file_size: u64 = u64::try_from(bytes.len())
      .map_err(|_| XrfError::new_texture_processing_error("DDS input exceeds the supported size range"))?;
    let dds: Dds = Dds::read(&mut Cursor::new(bytes))
      .map_err(|error| XrfError::new_texture_processing_error(format!("Failed to read DDS from memory: {error}.")))?;

    Self::from_parsed(dds, file_size)
  }

  pub fn encode_rgba(image: &RgbaImage, options: DdsEncodeOptions) -> XrfResult<Self> {
    let dds: Dds = dds_from_image(image, options.format, options.quality, options.mipmaps)
      .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;
    let data_size: u64 = u64::try_from(dds.data.len())
      .map_err(|_| XrfError::new_texture_processing_error("Encoded DDS exceeds the supported size range"))?;
    let metadata_size: u64 = if dds.header10.is_some() {
      DX10_HEADER_SIZE
    } else {
      HEADER_SIZE
    };
    let file_size: u64 = metadata_size
      .checked_add(data_size)
      .ok_or_else(|| XrfError::new_texture_processing_error("Encoded DDS size overflowed"))?;

    Ok(Self {
      dds,
      file_size,
      metadata_size,
    })
  }

  pub fn metadata(&self) -> DdsMetadata {
    DdsMetadata::from_dds(&self.dds, self.file_size, self.metadata_size)
  }

  /// Decode one mip level to RGBA, for whatever wants pixels rather than the file.
  ///
  /// Two decoders, in order. `image_dds` owns every block compressed layout and the packings it has names for; what it
  /// refuses goes to [`decode_uncompressed`], which expands a packing from the channel masks its header declares.
  /// Between them they cover every layout in the reference trees - 28,606 files across `gamedata`, `gamedata-anomaly`,
  /// `gamedata-coc`, `gamedata-cop-ee`, `gamedata-cs` and `gamedata-openxray-gunslinger`:
  ///
  /// | Layout | Files | Decoded by |
  /// | --- | --- | --- |
  /// | `DXT1`, `DXT3`, `DXT5` (BC1-BC3) | 27,314 | `image_dds` |
  /// | `A8R8G8B8`, `R8G8B8` | 889 | `image_dds` |
  /// | `A8B8G8R8` | 62 | `image_dds` |
  /// | `BC7_UNorm`, `R8G8B8A8_UNorm_sRGB` (DX10) | 31 | `image_dds` |
  /// | `ATI2` (BC5) | 5 | `image_dds` |
  /// | `A8`, alpha only | 274 | mask expansion |
  /// | `R5G6B5` | 15 | mask expansion |
  /// | 16bpp alpha-luminance | 9 | mask expansion |
  /// | `X8R8G8B8`, no alpha mask | 4 | mask expansion |
  /// | `L8`, luminance | 3 | mask expansion |
  ///
  /// # Errors
  ///
  /// Returns an error when neither decoder reads the layout, when the file is a cubemap or volume, or when the
  /// requested level is not in the file.
  pub fn decode_rgba(&self, mipmap_level: u32) -> XrfResult<RgbaImage> {
    match image_dds::image_from_dds(&self.dds, mipmap_level) {
      Ok(image) => Ok(image),
      Err(error) => decode_uncompressed(&self.dds, mipmap_level).map_err(|fallback| {
        XrfError::new_texture_processing_error(format!(
          "Failed to convert DDS to RGBA image: {error}, and it is no expandable packing either: {fallback}"
        ))
      }),
    }
  }

  pub fn to_png(&self) -> XrfResult<DdsPng> {
    let image: RgbaImage = self.decode_rgba(0)?;
    let mut bytes: Vec<u8> = Vec::new();

    PngEncoder::new(bytes.by_ref())
      .write_image(image.as_raw(), image.width(), image.height(), ExtendedColorType::Rgba8)
      .map_err(|error| XrfError::new_texture_processing_error(format!("Failed to encode DDS as PNG: {error}.")))?;

    Ok(DdsPng {
      width: image.width(),
      height: image.height(),
      bytes,
    })
  }

  pub fn write_to_path(&self, path: &Path) -> XrfResult {
    self
      .dds
      .write(&mut BufWriter::new(File::create(path)?))
      .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

    Ok(())
  }

  pub fn write_to_bytes(&self) -> XrfResult<Vec<u8>> {
    let mut bytes: Vec<u8> = Vec::new();

    self
      .dds
      .write(&mut bytes)
      .map_err(|error| XrfError::new_texture_processing_error(error.to_string()))?;

    Ok(bytes)
  }

  pub fn is_xray_compatible(&self) -> bool {
    if let Some(header10) = &self.dds.header10 {
      Self::is_xray_supported_format(header10.dxgi_format)
    } else if let Some(format) = DxgiFormat::try_from_pixel_format(&self.dds.header.spf) {
      Self::is_xray_supported_format(format)
    } else {
      true
    }
  }

  pub fn is_xray_supported_format(format: DxgiFormat) -> bool {
    matches!(
      format,
      DxgiFormat::BC1_UNorm
        | DxgiFormat::BC1_UNorm_sRGB
        | DxgiFormat::BC2_UNorm
        | DxgiFormat::BC2_UNorm_sRGB
        | DxgiFormat::BC3_UNorm
        | DxgiFormat::BC3_UNorm_sRGB
    )
  }

  fn from_parsed(dds: Dds, file_size: u64) -> XrfResult<Self> {
    let data_size: u64 = u64::try_from(dds.data.len())
      .map_err(|_| XrfError::new_texture_processing_error("DDS payload exceeds the supported size range"))?;
    let metadata_size: u64 = file_size
      .checked_sub(data_size)
      .ok_or_else(|| XrfError::new_texture_processing_error("DDS payload is larger than its source"))?;

    Ok(Self {
      dds,
      file_size,
      metadata_size,
    })
  }
}

#[cfg(test)]
mod tests {
  use ddsfile::{AlphaMode, D3D10ResourceDimension, Dds, DxgiFormat, NewDxgiParams};
  use image::RgbaImage;
  use image_dds::{ImageFormat, Mipmaps, Quality};
  use xrf_test_utils::utils::write_generated_test_resource;

  use super::{DdsEncodeOptions, DdsFile};

  fn encoded_file(width: u32, height: u32, mipmaps: Mipmaps) -> DdsFile {
    DdsFile::encode_rgba(
      &RgbaImage::new(width, height),
      DdsEncodeOptions::new(ImageFormat::BC3RgbaUnorm, Quality::Slow, mipmaps),
    )
    .expect("expect the DDS to encode")
  }

  fn dx10_file(format: DxgiFormat) -> DdsFile {
    let dds: Dds = Dds::new_dxgi(NewDxgiParams {
      height: 4,
      width: 4,
      depth: None,
      format,
      mipmap_levels: None,
      array_layers: None,
      caps2: None,
      is_cubemap: false,
      resource_dimension: D3D10ResourceDimension::Texture2D,
      alpha_mode: AlphaMode::Unknown,
    })
    .expect("expect the test DDS to be constructible");
    let mut bytes: Vec<u8> = Vec::new();

    dds.write(&mut bytes).expect("expect the test DDS to serialize");

    DdsFile::read_from_bytes(&bytes).expect("expect the test DDS to parse")
  }

  #[test]
  fn keeps_dimensions_that_are_not_multiples_of_four() {
    let metadata = encoded_file(1023, 1020, Mipmaps::Disabled).metadata();

    assert_eq!((metadata.width, metadata.height), (1023, 1020));
  }

  #[test]
  fn writes_the_requested_mip_chain() {
    let generated = encoded_file(1023, 1020, Mipmaps::GeneratedAutomatic).metadata();
    let flat = encoded_file(1023, 1020, Mipmaps::Disabled).metadata();

    assert_eq!(generated.mipmap_levels, 10);
    assert_eq!(flat.mipmap_levels, 1);
  }

  #[test]
  fn reads_the_same_metadata_from_the_header_alone() {
    // The header-only read exists so a survey can answer format questions without reading gigabytes of payload. It is
    // only worth having if it agrees with the full read, mip count and payload size included.
    let encoded: DdsFile = encoded_file(256, 128, Mipmaps::GeneratedAutomatic);
    let bytes: Vec<u8> = encoded.write_to_bytes().expect("expect DDS bytes");
    let path = write_generated_test_resource("xrf-dds/header-only.dds", &bytes).expect("expect scratch DDS");

    assert_eq!(
      DdsFile::read_metadata_from_path(&path).expect("expect the header to parse"),
      DdsFile::read_from_path(&path).expect("expect path to parse").metadata()
    );
  }

  #[test]
  fn reads_a_dx10_header_without_its_payload() {
    // A DX10 header is twenty bytes longer, so the payload size derivation has to account for it.
    let file: DdsFile = dx10_file(DxgiFormat::BC3_UNorm);
    let bytes: Vec<u8> = file.write_to_bytes().expect("expect DDS bytes");
    let path = write_generated_test_resource("xrf-dds/header-only-dx10.dds", &bytes).expect("expect scratch DDS");

    let metadata = DdsFile::read_metadata_from_path(&path).expect("expect the header to parse");

    assert_eq!(metadata.metadata_size, 148);
    assert_eq!(metadata, file.metadata());
  }

  #[test]
  fn reads_the_same_metadata_from_bytes_and_path() {
    let encoded: DdsFile = encoded_file(64, 32, Mipmaps::Disabled);
    let bytes: Vec<u8> = encoded.write_to_bytes().expect("expect DDS bytes");
    let path = write_generated_test_resource("xrf-dds/read-path.dds", &bytes).expect("expect scratch DDS");

    assert_eq!(
      DdsFile::read_from_bytes(&bytes)
        .expect("expect bytes to parse")
        .metadata(),
      DdsFile::read_from_path(&path).expect("expect path to parse").metadata()
    );
  }

  #[test]
  fn transcodes_the_base_mip_to_png() {
    let png = encoded_file(16, 8, Mipmaps::Disabled)
      .to_png()
      .expect("expect the DDS to transcode");

    assert_eq!((png.width, png.height), (16, 8));
    assert_eq!(&png.bytes[..8], b"\x89PNG\r\n\x1a\n");
  }

  #[test]
  fn identifies_xray_compatible_formats() {
    assert!(dx10_file(DxgiFormat::BC1_UNorm_sRGB).is_xray_compatible());
    assert!(dx10_file(DxgiFormat::BC3_UNorm).is_xray_compatible());
    assert!(!dx10_file(DxgiFormat::BC4_UNorm).is_xray_compatible());
  }

  #[test]
  fn rejects_malformed_bytes() {
    assert!(DdsFile::read_from_bytes(b"not a DDS").is_err());
  }
}
