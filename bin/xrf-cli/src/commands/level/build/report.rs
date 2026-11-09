use serde::Serialize;

/// What a file `level build` wrote replaced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum LevelBuildPrevious {
  /// The destination held no such file.
  New,
  /// The destination held the same bytes.
  Same,
  /// The destination held other bytes, now replaced.
  Changed,
}

/// One file `level build` wrote.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelBuildFile {
  pub(crate) name: String,
  pub(crate) size: usize,
  pub(crate) previous: LevelBuildPrevious,
}

/// What `level build` wrote, as its JSON report says it.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LevelBuildReport {
  pub(crate) source: String,
  pub(crate) destination: String,
  pub(crate) files: Vec<LevelBuildFile>,
}
