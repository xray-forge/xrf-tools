use xrf_app_core::TauriResult;
use xrf_audio::{AudioDeviceDescription, AudioDevices};

/// Name the output devices the system offers now, the default one marked, for the settings to offer.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "list_devices")]
pub async fn audio_list_devices() -> TauriResult<Vec<AudioDeviceDescription>> {
  tauri::async_runtime::spawn_blocking(AudioDevices::list)
    .await
    .map_err(|error| format!("Listing the output devices failed: {error}"))?
    .map_err(|error| error.to_string())
}
