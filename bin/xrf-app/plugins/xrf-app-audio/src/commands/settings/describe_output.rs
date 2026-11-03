use tauri::State;
use xrf_audio::AudioOutputStatus;

use crate::engine::AudioEngineState;

/// Say which device sound plays on now, or why none does.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_output")]
pub fn audio_describe_output(engine: State<'_, AudioEngineState>) -> AudioOutputStatus {
  engine.get_output_status()
}
