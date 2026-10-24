use std::path::{Path, PathBuf};

use image::imageops::FilterType;
use image::{RgbaImage, imageops};
use xrf_app_core::TauriResult;
use xrf_dds::{DdsFile, DdsMipChain};
use xrf_renderer::CubeAssembly;
use xrf_utils::write_file_staged;

use crate::viewer::ViewerImage;
use crate::viewer::cube::{CubeExportFormat, CubeExportRequest};

/// A cubemap's capture written again: its faces resized, as one cube DDS, six uncompressed TGAs or six PNGs.
pub struct CubeExporter;

impl CubeExporter {
  /// Each face's name as six PNGs name it, in the engine's order `+x -x +y -y +z -z`.
  pub const FACE_NAMES: [&str; 6] = ["px", "nx", "py", "ny", "pz", "nz"];

  /// The files the export `request` asks for: one DDS at its target, or a file a face in its target folder, the TGAs
  /// `1.tga` to `6.tga` as `CDemoRecord` names its faces and the PNGs the capture's name and the face's.
  pub fn list_paths(request: &CubeExportRequest) -> Vec<PathBuf> {
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

  /// Writes the export `request` asks for, answering the files written.
  ///
  /// # Errors
  ///
  /// Returns an error when the capture cannot be read or split, or the export cannot be encoded or written.
  pub fn export(request: &CubeExportRequest) -> TauriResult<Vec<PathBuf>> {
    let CubeExportRequest {
      path: source,
      format,
      size,
      ..
    } = request;
    let cross: RgbaImage = ViewerImage::read_capture(source)?;
    let faces: Vec<RgbaImage> = CubeAssembly::split(&cross).map_err(|error| error.to_string())?;
    let captured: u32 = cross.width() / 4;
    // A DDS takes whole blocks, which every face size offered already is.
    let is_blocked: bool = matches!(format, CubeExportFormat::Dds(_));
    let (side, _): (u32, u32) = ViewerImage::get_export_size((captured, captured), *size, is_blocked);
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
        let (encoding, mipmaps) = export.to_encoding();
        let target: &Path = &paths[0];
        let dds: DdsFile = faces
          .iter()
          .map(|face| DdsMipChain::build(face, mipmaps))
          .collect::<Result<Vec<DdsMipChain>, _>>()
          .and_then(|chains| encoding.encode_cube(&chains))
          .map_err(|error| format!("Failed to encode cubemap '{}': {error}", target.display()))?;
        let encoded: Vec<u8> = dds
          .write_to_bytes()
          .map_err(|error| format!("Failed to encode cubemap '{}': {error}", target.display()))?;

        write_file_staged(target, &encoded)
          .map_err(|error| format!("Failed to write cubemap '{}': {error}", target.display()))?;
      }
      CubeExportFormat::Tga | CubeExportFormat::Png => {
        // Every face encoded before the first is written, so a failure leaves no half set.
        let encoded: Vec<Vec<u8>> = faces
          .iter()
          .zip(&paths)
          .map(|(face, target)| match format {
            CubeExportFormat::Tga => ViewerImage::encode_tga(face, target),
            _ => ViewerImage::encode_png(face, target),
          })
          .collect::<TauriResult<_>>()?;

        for (target, encoded) in paths.iter().zip(encoded) {
          write_file_staged(target, &encoded)
            .map_err(|error| format!("Failed to write cubemap face '{}': {error}", target.display()))?;
        }
      }
    }

    Ok(paths)
  }
}
