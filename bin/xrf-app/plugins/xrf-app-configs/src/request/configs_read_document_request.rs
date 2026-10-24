use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;

/// Which config of which open a reader wants.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsReadDocumentRequest {
  /// The open this read is addressed to; a read naming one closed is refused rather than answered.
  pub session_id: SessionId,
  /// Engine identity of the config to read.
  pub path: String,
}
