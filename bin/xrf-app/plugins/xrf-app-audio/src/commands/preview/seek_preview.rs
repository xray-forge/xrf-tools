use std::time::Duration;

use tauri::State;
use xrf_app_core::TauriResult;

use crate::preview::{AudioTransportId, PreviewTransports};

/// Move the preview to `position` milliseconds in, clamped to the sound: a playing preview plays on from there, any
/// other holds there paused.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "seek_preview")]
pub fn audio_seek_preview(
  transports: State<'_, PreviewTransports>,
  transport: AudioTransportId,
  position: u64,
) -> TauriResult<()> {
  transports.seek(transport, Duration::from_millis(position))
}
