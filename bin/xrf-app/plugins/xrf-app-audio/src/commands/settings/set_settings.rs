use tauri::State;
use xrf_app_core::TauriResult;
use xrf_audio::AudioServer;

use crate::focus::AudioFocus;
use crate::settings::{AudioSettings, AudioSettingsStore};

/// Play with `settings` from now on and keep them for the next run, answering them as kept, every volume in range.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "set_settings")]
pub fn audio_set_settings(
  store: State<'_, AudioSettingsStore>,
  server: State<'_, AudioServer>,
  focus: State<'_, AudioFocus>,
  settings: AudioSettings,
) -> TauriResult<AudioSettings> {
  let previous: AudioSettings = store.get();
  // A preference the disk refused still holds for this run.
  let kept: TauriResult<AudioSettings> = store.set(&settings);
  let current: AudioSettings = store.get();

  if current.output != previous.output
    && let Some(output) = server.get_output()
  {
    output.choose(current.output);
  }

  focus.apply(&server, &store);

  kept
}
