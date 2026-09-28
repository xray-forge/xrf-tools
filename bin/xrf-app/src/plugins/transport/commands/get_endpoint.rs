use tauri::State;

use crate::core::transport::TransportEndpoint;

/// Where the transport listens and the token it expects, which the frontend asks once.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_endpoint"))]
#[tauri::command(rename = "get_endpoint")]
pub fn transport_get_endpoint(endpoint: State<'_, TransportEndpoint>) -> TransportEndpoint {
  TransportEndpoint::clone(&endpoint)
}
