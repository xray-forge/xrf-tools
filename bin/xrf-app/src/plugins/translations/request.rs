use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

/// What a translation verification was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationsVerifyRequest {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
  /// Language the check is about.
  pub language: String,
}

/// What a translation formatting run, or a check of one, was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationsFormatRequest {
  /// Project directory to format.
  pub directory: PathBuf,
  /// Line endings to write, or nothing to keep what each file already uses.
  pub line_endings: Option<String>,
}
