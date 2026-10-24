use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;
use xrf_translation::TranslationProjectMode;
use xrf_vfs::XrayRoots;

/// Inputs and caller-owned identity of one translations opening.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct TranslationsOpenRequest {
  pub session_id: SessionId,
  pub roots: XrayRoots,
  pub mode: TranslationProjectMode,
  pub prefix: Option<String>,
}
