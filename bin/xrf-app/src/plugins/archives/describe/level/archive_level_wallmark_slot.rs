use serde::Serialize;
use xrf_db::LevelWallmarkSlot;
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// One material of a level's baked decals, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelWallmarkSlot {
  /// The blender the decals draw through, which names a definition inside `shaders.xr` rather than a file.
  pub shader: String,
  /// The texture they draw with, which does name a file. Absent for a slot holding nothing, because the exporter
  /// writes no names for one.
  pub texture: Option<ArchiveReference>,
  pub marks: usize,
  /// Vertices across the slot's decals, which is what it costs to draw.
  pub vertices: usize,
}

impl ArchiveLevelWallmarkSlot {
  /// Every slot of a level, with the texture each names resolved.
  pub fn of_all(source: &ArchiveDescribeSource, slots: &[LevelWallmarkSlot]) -> Vec<Self> {
    slots.iter().map(|slot| Self::of(source, slot)).collect()
  }

  /// One slot, taken over the decals it holds.
  fn of(source: &ArchiveDescribeSource, slot: &LevelWallmarkSlot) -> Self {
    Self {
      shader: slot.shader.clone(),
      texture: (!slot.texture.is_empty()).then(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &slot.texture)),
      marks: slot.marks.len(),
      vertices: slot.marks.iter().map(|mark| mark.vertices.len()).sum(),
    }
  }
}
