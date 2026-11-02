use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;

/// Which resolved root a reader wants, of which open.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsListRequest {
  /// The open this read is addressed to; a read naming one closed is refused rather than answered.
  pub session_id: SessionId,
  /// Engine identity of the entry point to resolve.
  pub entry: String,
}
