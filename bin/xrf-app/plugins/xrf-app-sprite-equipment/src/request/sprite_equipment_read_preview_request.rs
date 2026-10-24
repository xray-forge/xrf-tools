use serde::Deserialize;
use xrf_app_core::session::SessionId;

/// Which held sheet to read the picture of.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SpriteEquipmentReadPreviewRequest {
  pub session_id: SessionId,
}
