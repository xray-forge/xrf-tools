use tauri::http::Result as HttpResult;
use tauri::http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CACHE_CONTROL, CONTENT_LENGTH, CONTENT_TYPE, REFERER};
use tauri::http::{Request, Response};
use tauri::{Manager, Runtime, UriSchemeContext};

use crate::core::session::SessionId;
use crate::plugins::sprite_equipment::state::EquipmentSpriteState;

/// Serves bytes from exactly the sprite identified by the URL, after releasing the session lock.
pub fn get_sprite_stream_response<R: Runtime>(
  context: UriSchemeContext<R>,
  request: &Request<Vec<u8>>,
) -> HttpResult<Response<Vec<u8>>> {
  let state = context.app_handle().state::<EquipmentSpriteState>();
  sprite_response(&state, request)
}

pub(super) fn sprite_response(
  state: &EquipmentSpriteState,
  request: &Request<Vec<u8>>,
) -> HttpResult<Response<Vec<u8>>> {
  let uri = percent_encoding::percent_decode(request.uri().path().as_bytes()).decode_utf8_lossy();

  let Some((id, name)) = uri.trim_matches('/').rsplit_once('/') else {
    return Response::builder().status(404).body(Vec::new());
  };

  let opened = id.parse::<SessionId>().ok().and_then(|id| state.require(id).ok());

  let Some(opened) = opened.filter(|opened| opened.metadata.name == name) else {
    return Response::builder().status(404).body(Vec::new());
  };

  let mut response = Response::builder();

  if let Some(referer) = request.headers().get(REFERER).and_then(|header| header.to_str().ok()) {
    response = response.header(ACCESS_CONTROL_ALLOW_ORIGIN, referer.trim_matches('/'));
  }

  response
    .header(CACHE_CONTROL, "no-store")
    .header(CONTENT_TYPE, "image/png")
    .header(CONTENT_LENGTH, opened.preview.len())
    .body(opened.preview.clone())
}
