use serde::Serialize;
use xrf_level::LevelShaderEntry;
use xrf_shaders::ShaderLibraryFile;
use xrf_vfs::XrayAssetType;

use crate::describe::description::ArchiveReference;
use crate::describe::level::bundle::{ArchiveLevelEntry, ArchiveLevelShader};
use crate::describe::source::ArchiveDescribeSource;

/// One row of the level's shader table.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelSurface {
  pub index: usize,
  pub entry: ArchiveLevelEntry,
}

impl ArchiveLevelSurface {
  /// Every row of a shader table, with its textures resolved and its shader asked of the library.
  pub fn of_all(
    source: &ArchiveDescribeSource,
    entries: &[LevelShaderEntry],
    library: Option<&ShaderLibraryFile>,
  ) -> Vec<Self> {
    entries
      .iter()
      .enumerate()
      .map(|(index, entry)| Self {
        index,
        entry: match entry {
          LevelShaderEntry::Empty => ArchiveLevelEntry::Skipped,
          LevelShaderEntry::Malformed(raw) => ArchiveLevelEntry::Unusable { raw: raw.clone() },
          LevelShaderEntry::Reference(reference) => ArchiveLevelEntry::Drawn {
            shader: ArchiveLevelShader::of(&reference.shader, library),
            textures: reference
              .textures
              .iter()
              .map(|texture| ArchiveReference::resolve(source, XrayAssetType::Dds, texture))
              .collect(),
          },
        },
      })
      .collect()
  }
}
