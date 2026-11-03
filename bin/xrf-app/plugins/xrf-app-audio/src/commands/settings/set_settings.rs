use tauri::State;
use xrf_app_core::TauriResult;

use crate::engine::AudioEngineState;
use crate::settings::{AudioSettings, AudioSettingsStore};

/// Play with `settings` from now on and keep them for the next run, answering them as kept, every volume in range.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "set_settings")]
pub fn audio_set_settings(
  store: State<'_, AudioSettingsStore>,
  engine: State<'_, AudioEngineState>,
  settings: AudioSettings,
) -> TauriResult<AudioSettings> {
  let kept: TauriResult<AudioSettings> = store.set(&settings);

  // A preference the disk refused still holds for this run.
  engine.apply(store.get());

  kept
}
