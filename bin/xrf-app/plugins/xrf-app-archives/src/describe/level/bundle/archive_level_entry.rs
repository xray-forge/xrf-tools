use serde::Serialize;

use crate::describe::description::ArchiveReference;
use crate::describe::level::bundle::ArchiveLevelShader;

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
