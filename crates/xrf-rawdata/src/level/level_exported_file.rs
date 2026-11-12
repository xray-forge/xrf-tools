use serde::Serialize;

/// What a file [`LevelSceneExporter::write_to_directory`](crate::LevelSceneExporter::write_to_directory) wrote
/// replaced.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelExportedPrevious {
  /// The directory held no such file.
  New,
  /// The directory held the same bytes.
  Same,
  /// The directory held other bytes, now replaced.
  Changed,
}

/// One level file an export wrote.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelExportedFile {
  pub name: String,
  pub size: usize,
  pub previous: LevelExportedPrevious,
}
