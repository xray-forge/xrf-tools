use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;

/// Identifies a dialog within one committed project.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct DialogsReadRequest {
  pub session_id: SessionId,
  pub logical_path: String,
  pub id: String,
  pub language: Option<String>,
}
