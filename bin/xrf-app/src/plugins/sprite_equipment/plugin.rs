use tauri::http::header::CONTENT_TYPE;
use tauri::http::{Response, StatusCode};
use tauri::plugin::TauriPlugin;
use tauri::{Manager, Runtime};

use crate::plugins::sprite_equipment::state::EquipmentSpriteState;
use crate::plugins::sprite_equipment::stream::get_sprite_stream_response;

pub struct SpriteEquipmentPlugin {}

impl SpriteEquipmentPlugin {
  pub const NAME: &'static str = crate::ipc::registry::sprite_equipment::NAME;

  pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new(Self::NAME)
      .setup(|application, _| {
        application.manage(EquipmentSpriteState::new());

        Ok(())
      })
      .register_uri_scheme_protocol("stream", move |context, request| {
        get_sprite_stream_response(context, &request).unwrap_or_else(|error| {
          log::warn!("Failed to handle stream protocol request: {}", error);

          Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .header(CONTENT_TYPE, "text/plain")
            .body(error.to_string().as_bytes().to_vec())
            .unwrap()
        })
      })
      .invoke_handler(crate::core::logging::warn_on_unhandled_command(
        Self::NAME,
        crate::ipc::registry::sprite_equipment::handler(),
      ))
      .build()
  }

  #[cfg(feature = "typescript-bindings")]
  pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
    crate::ipc::registry::sprite_equipment::specta_builder()
  }
}
