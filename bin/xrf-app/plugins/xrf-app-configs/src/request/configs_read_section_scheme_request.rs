use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;

/// Which section of a resolved root a reader wants explained.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsReadSectionSchemeRequest {
  /// The open this read is addressed to; a read naming one closed is refused rather than answered.
  pub session_id: SessionId,
  /// Engine identity of the entry point the section belongs to.
  pub entry: String,
  /// The section to explain, as the index named it.
  pub section: String,
}
