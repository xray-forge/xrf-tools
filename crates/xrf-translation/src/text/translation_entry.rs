use xrf_collections::OrderedMap;

use crate::text::translation_variant::TranslationVariant;

/// One id's text in each language that has it. A missing language is a gap the engine fills with the id itself, so
/// absence is meaningful and is not the same as an empty string.
pub type TranslationEntry = OrderedMap<String, Option<TranslationVariant>>;
