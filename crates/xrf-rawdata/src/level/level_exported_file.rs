use serde::Serialize;

/// What a file [`LevelSceneExporter::write_to_directory`](crate::LevelSceneExporter::write_to_directory) wrote
/// replaced.
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
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelExportedFile {
  pub name: String,
  pub size: usize,
  pub previous: LevelExportedPrevious,
}
