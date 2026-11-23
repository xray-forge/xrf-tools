use serde::{Deserialize, Serialize};
use xrf_collections::OrderedMap;

use crate::project::describe::translation_source::TranslationSource;
use crate::text::translation_entry::TranslationEntry;

/// One logical translation file, and where each language's copy of it lives.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationFile {
  /// Language to the source holding it. A JSON source lists every language it carries against the
  /// same one.
  ///
  /// Editability is per language rather than per file: an installation layered under a loose tree can
  /// serve one language from a `.db` volume and the next from disk.
  #[cfg_attr(
    feature = "typescript-bindings",
    specta(type = std::collections::HashMap<String, TranslationSource>)
  )]
  pub sources: OrderedMap<String, TranslationSource>,
  #[cfg_attr(
    feature = "typescript-bindings",
    specta(type = std::collections::HashMap<String, std::collections::HashMap<String, Option<crate::text::translation_variant::TranslationVariant>>>)
  )]
  pub entries: OrderedMap<String, TranslationEntry>,
}

impl TranslationFile {
  /// Whether every language's copy of this file could be written back.
  ///
  /// Not on the wire. A caller edits one language at a time, so what it needs is whether *that*
  /// language's source is loose, which `sources` already says; a rolled-up per-file flag would be the
  /// weaker answer to the narrower question.
  pub fn is_editable(&self) -> bool {
    !self.sources.is_empty() && self.sources.values().all(TranslationSource::is_editable)
  }
}
