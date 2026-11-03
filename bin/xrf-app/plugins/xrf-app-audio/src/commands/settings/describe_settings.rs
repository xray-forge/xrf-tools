use tauri::State;

use crate::settings::{AudioSettings, AudioSettingsStore};

/// Say how the application plays sound: its device, volumes and background mute.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_settings")]
pub fn audio_describe_settings(store: State<'_, AudioSettingsStore>) -> AudioSettings {
  store.get()
}
