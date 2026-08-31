use std::sync::MutexGuard;

use tauri::http::Result as HttpResult;
use tauri::http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_LENGTH, CONTENT_TYPE, REFERER};
use tauri::http::response::Builder;
use tauri::http::{Request, Response};
use tauri::{AppHandle, Manager, Runtime, State, UriSchemeContext};

use crate::plugins::sprite_equipment::state::EquipmentSpriteState;

pub fn get_sprite_stream_response<R: Runtime>(
  context: UriSchemeContext<R>,
  request: &Request<Vec<u8>>,
) -> HttpResult<Response<Vec<u8>>> {
  let handle: &AppHandle<R> = context.app_handle();
  let sprite_equipment_state: State<EquipmentSpriteState> = handle.state::<EquipmentSpriteState>();

  let sprite_lock: MutexGuard<Option<String>> = sprite_equipment_state.equipment_sprite_name.lock().unwrap();
  let preview_lock: MutexGuard<Option<Vec<u8>>> = sprite_equipment_state.equipment_sprite_preview.lock().unwrap();

  let preview: Option<&Vec<u8>> = preview_lock.as_ref();
  let sprite_name: Option<&String> = sprite_lock.as_ref();

  if let (Some(preview), Some(sprite_name)) = (preview, sprite_name) {
    let uri: String = percent_encoding::percent_decode(request.uri().path().as_bytes())
      .decode_utf8_lossy()
      .to_string();

    if !uri.ends_with(&format!("/{}", sprite_name)) {
      log::info!("Incorrect asset request: {uri}");

      return Response::builder().status(404).body(Vec::new());
    }

    let mut response: Builder = Response::builder();

    if let Some(referer) = request.headers().get(REFERER).map(|header| header.to_str().unwrap()) {
      response = response.header(ACCESS_CONTROL_ALLOW_ORIGIN, referer.trim_matches('/'))
    }

    response
      .header(CONTENT_TYPE, "image/png")
      .header(CONTENT_LENGTH, preview.len())
      .body(preview.clone())
  } else {
    log::info!("Incorrect asset request while not existing");

    Response::builder().status(404).body(Vec::new())
  }
}
