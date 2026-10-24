use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_translation::TranslationLanguage;
use xrf_vfs::XrayRoots;

/// What an import was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationsParseRequest {
  /// Roots holding the raw XML, read through the VFS so an installation imports like a loose tree.
  pub roots: XrayRoots,
  /// The language every entry this run reads is filed under. Never `all`.
  pub language: TranslationLanguage,
  /// Where inside those roots to look, or nothing to let the run resolve it.
  pub prefix: Option<String>,
  /// Directory the JSON sources are written to, which may already hold some.
  pub output_dir: PathBuf,
  /// Restrict the run to one table, by the file name it has in the scope.
  pub file: Option<String>,
  /// Let incoming text replace existing text that differs, instead of keeping what is there.
  pub is_overwrite: bool,
  /// Do everything except write, so a caller can see what a run would change.
  pub is_dry_run: bool,
}
