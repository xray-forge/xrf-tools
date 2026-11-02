use bytes::Bytes;
use tauri::{AppHandle, Manager};
use xrf_app_core::TauriResult;
use xrf_app_core::transport::TransportAnswer;

use crate::EquipmentSprites;
use crate::request::SpriteEquipmentReadPreviewRequest;

/// The sheet `session_id` holds, as the PNG it is drawn from, served as held.
pub(crate) async fn sprite_equipment_read_preview(
  app: AppHandle,
  request: SpriteEquipmentReadPreviewRequest,
) -> TauriResult<TransportAnswer> {
  let preview: Bytes = app
    .state::<EquipmentSprites>()
    .require(request.session_id)?
    .preview
    .clone();

  Ok(TransportAnswer::png(preview))
}
