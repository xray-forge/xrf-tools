use serde::{Deserialize, Serialize};

/// One `<string id="…"><text>…</text></string>` as the build emits it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename = "string")]
pub(crate) struct TranslationEntryCompiled {
  #[serde(rename = "@id")]
  pub id: String,
  pub text: String,
}
