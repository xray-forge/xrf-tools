use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{RgbaImage, imageops};
use xrf_app_core::TauriResult;
use xrf_dds::{DdsFile, DdsMipChain};
use xrf_renderer::CubeAssembly;
use xrf_utils::StagedFile;

use crate::viewer::ViewerImage;
use crate::viewer::export::cube::{CubeExportFormat, CubeExportRequest};

/// Re-exports a cubemap capture with resized faces as one cube DDS, six uncompressed TGAs or six PNGs.
pub(crate) struct CubeExporter;

impl CubeExporter {
  /// PNG face suffixes, in the engine's order `+x -x +y -y +z -z`.
  pub(crate) const FACE_NAMES: [&str; 6] = ["px", "nx", "py", "ny", "pz", "nz"];

  /// The files `request` writes: one DDS at its target, or one file per face in its target folder, `1.tga` to `6.tga`
  /// as `CDemoRecord` names them, or `<capture>_<face>.png`.
  pub(crate) fn list_paths(request: &CubeExportRequest) -> Vec<PathBuf> {
    let CubeExportRequest {
      path, target, format, ..
    } = request;

    match format {
      CubeExportFormat::Dds(_) => vec![target.clone()],
      CubeExportFormat::Tga => (1..=Self::FACE_NAMES.len())
        .map(|face| target.join(format!("{face}.tga")))
        .collect(),
      CubeExportFormat::Png => {
        let stem: String = path
          .file_stem()
          .map_or_else(|| "cube".to_string(), |stem| stem.to_string_lossy().into_owned());

        Self::FACE_NAMES
          .iter()
          .map(|face| target.join(format!("{stem}_{face}.png")))
          .collect()
      }
    }
  }

  /// Write the export `request` asks for, answering the files written.
  ///
  /// # Errors
  ///
  /// Returns an error when the capture cannot be read or split, or the export cannot be encoded or written.
  pub(crate) fn export(request: &CubeExportRequest) -> TauriResult<Vec<PathBuf>> {
    let CubeExportRequest {
      path: source,
      format,
      size,
      ..
    } = request;
    let cross: RgbaImage = ViewerImage::read_capture(source)?;
    let faces: Vec<RgbaImage> = CubeAssembly::split(&cross).map_err(|error| error.to_string())?;
    let captured: u32 = cross.width() / 4;
    // A DDS needs whole blocks; every offered face size already is.
    let (side, _): (u32, u32) = match format {
      CubeExportFormat::Dds(_) => ViewerImage::get_dds_export_size((captured, captured), *size),
      _ => ViewerImage::get_export_size((captured, captured), *size),
    };
    let faces: Vec<RgbaImage> = faces
      .into_iter()
      .map(|face| {
        if side == captured {
          face
        } else {
          imageops::resize(&face, side, side, FilterType::Lanczos3)
        }
      })
      .collect();
    let paths: Vec<PathBuf> = Self::list_paths(request);

    match format {
      CubeExportFormat::Dds(export) => {
        let target: &Path = &paths[0];
        let dds: DdsFile = faces
          .into_iter()
          .map(|face| DdsMipChain::build(face, export.to_mipmaps()))
          .collect::<Result<Vec<DdsMipChain>, _>>()
          .and_then(|chains| export.to_encoding().encode_cube(&chains))
          .map_err(|error| format!("Failed to encode cubemap '{}': {error}", target.display()))?;
        let encoded: Vec<u8> = dds
          .write_to_bytes()
          .map_err(|error| format!("Failed to encode cubemap '{}': {error}", target.display()))?;

        StagedFile::write(target, &encoded)
          .map_err(|error| format!("Failed to write cubemap '{}': {error}", target.display()))?;
      }
      CubeExportFormat::Tga | CubeExportFormat::Png => {
        // Encode every face before writing any, so a failure leaves no partial set.
        let encoded: Vec<Vec<u8>> = faces
          .iter()
          .zip(&paths)
          .map(|(face, target)| match format {
            CubeExportFormat::Tga => ViewerImage::encode_tga(face, target),
            _ => ViewerImage::encode_png(face, target),
          })
          .collect::<TauriResult<_>>()?;

        for (target, encoded) in paths.iter().zip(encoded) {
          StagedFile::write(target, &encoded)
            .map_err(|error| format!("Failed to write cubemap face '{}': {error}", target.display()))?;
        }
      }
    }

    Ok(paths)
  }
}
