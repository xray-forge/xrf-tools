use serde::{Deserialize, Serialize};

use crate::project::build::compiled::translation_entry_compiled::TranslationEntryCompiled;

/// A whole string table document, for serializing only.
///
/// Reading goes through the span reader instead, which keeps positions so an edit can splice the
/// original bytes. This shape exists to write a file the build generates from scratch.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(rename = "string_table")]
pub(crate) struct TranslationCompiledXml {
  pub string: Vec<TranslationEntryCompiled>,
}
