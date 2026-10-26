use indexmap::IndexMap;

use crate::text::translation_entry::TranslationEntry;

/// Every id in one source file.
pub type TranslationJson = IndexMap<String, TranslationEntry>;
