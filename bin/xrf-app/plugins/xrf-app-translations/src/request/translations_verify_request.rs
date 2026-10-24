use serde::{Deserialize, Serialize};
use xrf_translation::TranslationLanguage;
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
  /// Language the check is about, or `all`.
  pub language: TranslationLanguage,
}
