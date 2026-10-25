use serde::Serialize;
use xrf_level::LevelShaderEntry;
use xrf_shaders::ShaderLibraryFile;
use xrf_vfs::XrayAssetType;

use crate::describe::description::ArchiveReference;
use crate::describe::level::bundle::ArchiveLevelShader;
use crate::describe::source::ArchiveDescribeSource;

/// One row of the level's shader table.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelSurface {
  pub index: usize,
  pub entry: ArchiveLevelEntry,
}

/// What one row of the table holds, in the three shapes the renderer distinguishes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ArchiveLevelEntry {
  // An empty name, which the renderer skips. Every shipped level has exactly one, at row zero.
  Skipped,
  // No `/` to split on. The renderer dereferences its `strchr` result without a null check, so a row like this is a
  // crash rather than a surface that fails to draw. None of the 15,283 rows across the workspace trees is one.
  Unusable {
    raw: String,
  },
  Drawn {
    shader: ArchiveLevelShader,
    textures: Vec<ArchiveReference>,
  },
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
