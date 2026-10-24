use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;

/// Which sections of a resolved root a page wants.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsReadSectionsRequest {
  /// The open this read is addressed to; a read naming one closed is refused rather than answered.
  pub session_id: SessionId,
  /// Engine identity of the entry point the sections belong to.
  pub entry: String,
  /// Sections to read, as the index named them.
  pub names: Vec<String>,
}
