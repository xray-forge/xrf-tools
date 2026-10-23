//! A level map's capture written again: resized, flattened or not, as a PNG, an uncompressed TGA or a DDS, or its
//! annotations alone as their SVG.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::codecs::tga::TgaEncoder;
use image::imageops::FilterType;
use image::{ExtendedColorType, ImageEncoder, ImageFormat, ImageReader, Rgba, RgbaImage, imageops};
use xrf_dds::{DDS_BLOCK_SIZE, DdsEncoding, DdsFile, DdsMipChain, DdsMipFilter, DdsMipmaps};
use xrf_utils::write_file_staged;

use crate::core::types::TauriResult;
use crate::plugins::levels::viewer::LevelViewerFolder;
use crate::plugins::levels::viewer::annotations::annotation_raster::rasterise_annotations;
use crate::plugins::levels::viewer::map_export_format::MapExportFormat;
use crate::plugins::levels::viewer::map_export_request::{MapExportAnnotations, MapExportRequest};
use xrf_texture::{TextureEncodingFormat, TextureEncodingQuality};

/// An export's pixels: the longer side `long_side` at most the capture's, a DDS rounded to whole blocks.
///
/// The rounding resamples rather than pads: the game stretches the texture over the map's area, so it must cover it.
pub fn get_export_size((width, height): (u32, u32), long_side: u32, is_blocked: bool) -> (u32, u32) {
  let longest: u32 = width.max(height).max(1);
  let scale: f64 = f64::from(long_side.clamp(1, longest)) / f64::from(longest);
  let side = |pixels: u32| -> u32 {
    let scaled: f64 = (f64::from(pixels) * scale).round().max(1.0);

    if is_blocked {
      let block: f64 = f64::from(DDS_BLOCK_SIZE);

      ((scaled / block).round().max(1.0) * block) as u32
    } else {
      scaled as u32
    }
  };

  (side(width), side(height))
}

/// `image` laid over the backdrop colour `[red, green, blue]`, opaque where it was clear.
pub fn flatten_onto(image: &mut RgbaImage, [red, green, blue]: [u8; 3]) {
  for Rgba([r, g, b, a]) in image.pixels_mut() {
    let alpha: u16 = u16::from(*a);
    let over = |channel: u8, under: u8| -> u8 {
      ((u16::from(channel) * alpha + u16::from(under) * (255 - alpha) + 127) / 255) as u8
    };

    *r = over(*r, red);
    *g = over(*g, green);
    *b = over(*b, blue);
    *a = 255;
  }
}

/// A capture's PNG of the screenshots folder, however large: a map 16384 pixels a side takes a gigabyte decoded, twice
/// the decoder's own limit.
///
/// # Errors
///
/// Returns an error when it cannot be read or decoded.
pub fn read_capture(source: &Path) -> TauriResult<RgbaImage> {
  let bytes: Vec<u8> = fs::read(source).map_err(|error| format!("Failed to read '{}': {error}", source.display()))?;
  let mut reader: ImageReader<Cursor<Vec<u8>>> = ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);

  reader.no_limits();

  Ok(
    reader
      .decode()
      .map_err(|error| format!("Failed to decode '{}': {error}", source.display()))?
      .to_rgba8(),
  )
}

/// `image` as a PNG, to be written to `target`.
///
/// # Errors
///
/// Returns an error when it cannot be encoded.
pub fn encode_png(image: &RgbaImage, target: &Path) -> TauriResult<Vec<u8>> {
  let mut png: Vec<u8> = Vec::new();

  image
    .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
    .map_err(|error| format!("Failed to encode '{}': {error}", target.display()))?;

  Ok(png)
}

/// `image` as an uncompressed 32 bit TGA, as `CDemoRecord` writes its own, to be written to `target`.
///
/// # Errors
///
/// Returns an error when it cannot be encoded.
pub fn encode_tga(image: &RgbaImage, target: &Path) -> TauriResult<Vec<u8>> {
  let mut tga: Vec<u8> = Vec::new();

  TgaEncoder::new(&mut tga)
    .disable_rle()
    .write_image(image.as_raw(), image.width(), image.height(), ExtendedColorType::Rgba8)
    .map_err(|error| format!("Failed to encode '{}': {error}", target.display()))?;

  Ok(tga)
}

/// The encoder a DDS export writes `format` at `quality` with, and its mipmaps: the whole chain, reduced with the X-Ray
/// converter's Kaiser kernel, or none.
pub fn get_dds_export(
  format: TextureEncodingFormat,
  quality: TextureEncodingQuality,
  is_mipmapped: bool,
) -> (DdsEncoding, DdsMipmaps) {
  let mipmaps: DdsMipmaps = if is_mipmapped {
    DdsMipmaps::Filtered(DdsMipFilter::Kaiser)
  } else {
    DdsMipmaps::Disabled
  };

  (
    DdsEncoding::new(format.to_candidate().to_image_format(), quality.to_quality()),
    mipmaps,
  )
}

/// Why an export other than the annotations alone is refused as an SVG.
fn refuse_svg() -> String {
  "Only a level map's annotations alone are exported as an SVG".to_string()
}

/// Writes the export `request` asks for, answering its pixels across and down.
///
/// # Errors
///
/// Returns an error when the capture cannot be read, or the export cannot be encoded or written.
pub fn export_map(request: &MapExportRequest) -> TauriResult<(u32, u32)> {
  let MapExportRequest {
    path: source,
    target,
    format,
    long_side,
    backdrop,
    annotations,
  } = request;
  let svg: Option<String> = annotations
    .map(|_| {
      let path: PathBuf = LevelViewerFolder::get_annotations_path(source);

      fs::read_to_string(&path).map_err(|error| format!("Failed to read annotations '{}': {error}", path.display()))
    })
    .transpose()?;

  // The annotations alone as they were drawn, a vector at any size.
  if let MapExportFormat::Svg = format {
    let (Some(MapExportAnnotations::Alone), Some(svg)) = (annotations, &svg) else {
      return Err(refuse_svg());
    };

    let (width, height): (u32, u32) =
      image::image_dimensions(source).map_err(|error| format!("Failed to read map '{}': {error}", source.display()))?;

    write_file_staged(target, svg.as_bytes())
      .map_err(|error| format!("Failed to write annotations '{}': {error}", target.display()))?;

    return Ok((width, height));
  }

  let capture: RgbaImage = read_capture(source)?;
  let is_blocked: bool = matches!(format, MapExportFormat::Dds { .. });
  let (width, height): (u32, u32) = get_export_size(capture.dimensions(), *long_side, is_blocked);
  let mut image: RgbaImage = if (width, height) == capture.dimensions() {
    capture
  } else {
    // Lanczos, as `CDemoRecord`'s level map resamples its frame.
    imageops::resize(&capture, width, height, FilterType::Lanczos3)
  };

  if let (Some(annotations), Some(svg)) = (annotations, &svg) {
    let drawn: RgbaImage = rasterise_annotations(svg, (width, height))?;

    match annotations {
      MapExportAnnotations::Over => imageops::overlay(&mut image, &drawn, 0, 0),
      MapExportAnnotations::Alone => image = drawn,
    }
  }

  if let Some(backdrop) = backdrop {
    flatten_onto(&mut image, *backdrop);
  }

  let encoded: Vec<u8> = match format {
    MapExportFormat::Png => encode_png(&image, target)?,
    MapExportFormat::Tga => encode_tga(&image, target)?,
    MapExportFormat::Svg => return Err(refuse_svg()),
    MapExportFormat::Dds {
      format,
      quality,
      is_mipmapped,
    } => {
      let (encoding, mipmaps): (DdsEncoding, DdsMipmaps) = get_dds_export(*format, *quality, *is_mipmapped);
      let dds: DdsFile = DdsMipChain::build(&image, mipmaps)
        .and_then(|chain| encoding.encode(&chain))
        .map_err(|error| format!("Failed to encode map '{}': {error}", target.display()))?;

      dds
        .write_to_bytes()
        .map_err(|error| format!("Failed to encode map '{}': {error}", target.display()))?
    }
  };

  // Encoded whole first, so a failed write leaves no partial file.
  write_file_staged(target, &encoded)
    .map_err(|error| format!("Failed to write map '{}': {error}", target.display()))?;

  Ok((width, height))
}

#[cfg(test)]
mod tests {
  use super::get_export_size;

  // The page sizes its preview by the same cases (`level-map-export.test.ts`).
  #[test]
  fn sizes_an_export_in_the_capture_s_shape_a_dds_in_whole_blocks() {
    assert_eq!(get_export_size((4096, 2048), 2048, false), (2048, 1024));
    assert_eq!(get_export_size((1000, 333), 500, false), (500, 167));
    assert_eq!(get_export_size((1000, 333), 500, true), (500, 168));
    assert_eq!(get_export_size((512, 512), 4096, true), (512, 512));
  }
}
