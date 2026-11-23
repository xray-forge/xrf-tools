use xrf_collections::OrderedMap;

use crate::text::translation_entry::TranslationEntry;

/// Every id in one source file.
pub type TranslationJson = OrderedMap<String, TranslationEntry>;
