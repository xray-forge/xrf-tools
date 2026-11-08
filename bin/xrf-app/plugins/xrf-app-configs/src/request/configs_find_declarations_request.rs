use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;

/// Where a reader wants to know a section, a field or a value is written, across the open project.
///
/// No way to say how names compare: that is the project's dialect's to decide, so the backend reads it from the project
/// rather than trusting a page to agree.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfigsFindDeclarationsRequest {
  /// The open this search is addressed to; a search naming one closed is refused rather than answered.
  pub session_id: SessionId,
  /// Section name without brackets or operation prefix.
  pub section: Option<String>,
  /// Field name without operation prefix.
  pub field: Option<String>,
  /// Text a written value must contain.
  pub value: Option<String>,
}
