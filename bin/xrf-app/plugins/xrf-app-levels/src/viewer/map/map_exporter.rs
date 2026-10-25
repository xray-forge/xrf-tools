use std::fs;
use std::path::PathBuf;

use image::imageops::FilterType;
use image::{RgbaImage, imageops};
use xrf_app_core::TauriResult;
use xrf_dds::{DdsFile, DdsMipChain};
use xrf_utils::StagedFile;

use crate::viewer::map::annotations::AnnotationCanvas;
use crate::viewer::map::{MapExportAnnotations, MapExportFormat, MapExportRequest};
use crate::viewer::{LevelViewerFolder, ViewerImage};

/// A level map's capture written again: resized, flattened or not, as a PNG, an uncompressed TGA or a DDS, or its
/// annotations alone as their SVG.
pub struct MapExporter;

impl MapExporter {
  /// Writes the export `request` asks for, answering its pixels across and down.
  ///
  /// # Errors
  ///
  /// Returns an error when the capture cannot be read, or the export cannot be encoded or written.
  pub fn export(request: &MapExportRequest) -> TauriResult<(u32, u32)> {
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
        return Err(Self::refuse_svg());
      };

      let (width, height): (u32, u32) = image::image_dimensions(source)
        .map_err(|error| format!("Failed to read map '{}': {error}", source.display()))?;

      StagedFile::write(target, svg.as_bytes())
        .map_err(|error| format!("Failed to write annotations '{}': {error}", target.display()))?;

      return Ok((width, height));
    }

    let capture: RgbaImage = ViewerImage::read_capture(source)?;
    let is_blocked: bool = matches!(format, MapExportFormat::Dds(_));
    let (width, height): (u32, u32) = ViewerImage::get_export_size(capture.dimensions(), *long_side, is_blocked);
    let mut image: RgbaImage = if (width, height) == capture.dimensions() {
      capture
    } else {
      // Lanczos, as `CDemoRecord`'s level map resamples its frame.
      imageops::resize(&capture, width, height, FilterType::Lanczos3)
    };

    if let (Some(annotations), Some(svg)) = (annotations, &svg) {
      let drawn: RgbaImage = AnnotationCanvas::rasterise(svg, (width, height))?;

      match annotations {
        MapExportAnnotations::Over => imageops::overlay(&mut image, &drawn, 0, 0),
        MapExportAnnotations::Alone => image = drawn,
      }
    }

    if let Some(backdrop) = backdrop {
      ViewerImage::flatten_onto(&mut image, *backdrop);
    }

    let encoded: Vec<u8> = match format {
      MapExportFormat::Png => ViewerImage::encode_png(&image, target)?,
      MapExportFormat::Tga => ViewerImage::encode_tga(&image, target)?,
      MapExportFormat::Svg => return Err(Self::refuse_svg()),
      MapExportFormat::Dds(export) => {
        let (encoding, mipmaps) = export.to_encoding();
        let dds: DdsFile = DdsMipChain::build(&image, mipmaps)
          .and_then(|chain| encoding.encode(&chain))
          .map_err(|error| format!("Failed to encode map '{}': {error}", target.display()))?;

        dds
          .write_to_bytes()
          .map_err(|error| format!("Failed to encode map '{}': {error}", target.display()))?
      }
    };

    // Encoded whole first, so a failed write leaves no partial file.
    StagedFile::write(target, &encoded)
      .map_err(|error| format!("Failed to write map '{}': {error}", target.display()))?;

    Ok((width, height))
  }

  /// Why an export other than the annotations alone is refused as an SVG.
  fn refuse_svg() -> String {
    "Only a level map's annotations alone are exported as an SVG".to_string()
  }
}
