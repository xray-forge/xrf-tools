use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;

/// A search for where the open project writes a section, a field or a value.
///
/// It carries no name comparison: the backend takes that from the project's dialect.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfigsFindDeclarationsRequest {
  /// The open project searched; a closed one is refused.
  pub session_id: SessionId,
  /// Section name without brackets or operation prefix.
  pub section: Option<String>,
  /// Field name without operation prefix.
  pub field: Option<String>,
  /// Text a written value must contain.
  pub value: Option<String>,
}
