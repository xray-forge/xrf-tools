use tauri::State;
use xrf_audio::{AudioOutputStatus, AudioServer};

/// Say which device sound plays on now, or why none does.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_output")]
pub(crate) fn audio_describe_output(server: State<'_, AudioServer>) -> AudioOutputStatus {
  server.get_output_status()
}
