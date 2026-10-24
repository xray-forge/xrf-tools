use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_translation::TranslationLanguage;
use xrf_vfs::XrayRoots;

/// What a build was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationsBuildRequest {
  /// Where the sources are read from, through the VFS.
  pub roots: XrayRoots,
  /// Where inside those roots to look, or nothing for the whole set.
  pub prefix: Option<String>,
  /// The language to build, or `all`.
  pub language: TranslationLanguage,
  /// Directory the string tables are written into, which is always a host path.
  pub output_dir: PathBuf,
  /// Whether to sort entries within each table.
  pub is_sorted: bool,
}
