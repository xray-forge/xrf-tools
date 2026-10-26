use std::path::{Path, PathBuf};

use xrf_dds::{DdsMipFilter, ImageFormat};
use xrf_thm::{ThmBumpChunk, ThmFile};

/// What a bump descriptor asks of the pair it builds, which `DXTCompressBump` reads off it: everything else of the
/// descriptor, its format and its mip settings included, is left at the SDK's own (`fmt0`, DXT5 with a box chain).
#[derive(Clone, Debug, PartialEq)]
pub struct BuildTextureBumpPair {
  /// `bump_virtual_height`, how deep the relief is derived at.
  pub virtual_height: f32,
  /// The texture whose normals replace the ones the height implies, `ext_normal_map_name`, when the descriptor names
  /// one.
  pub normal_map: Option<String>,
}

impl BuildTextureBumpPair {
  /// The layout both halves are written in, `fmt0.fmt`, whatever the descriptor's own format.
  pub const FORMAT: ImageFormat = ImageFormat::BC3RgbaUnorm;

  /// The kernel both halves are reduced with, `fmt0`'s default, whatever the descriptor's own filter.
  pub const MIP_FILTER: DdsMipFilter = DdsMipFilter::Box;

  /// The pair a bump descriptor asks for.
  pub fn of(file: &ThmFile) -> Self {
    Self {
      virtual_height: file
        .bump
        .as_ref()
        .map_or(ThmBumpChunk::DEFAULT_VIRTUAL_HEIGHT, |bump| bump.virtual_height),
      normal_map: file.ext_normal_map_name.clone().filter(|name| !name.is_empty()),
    }
  }

  /// The bump a build writing `destination` names, the destination without its extension, as `DXTCompressBump` takes
  /// its output name.
  pub fn bump_of(destination: &Path) -> PathBuf {
    destination.with_extension("")
  }
}
