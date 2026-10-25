use xrf_app_core::TauriResult;
use xrf_pack::ArchivePackConfig;

/// Hand back a packing configuration with nothing chosen yet.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "default_pack_config")]
pub async fn archives_default_pack_config() -> TauriResult<ArchivePackConfig> {
  Ok(ArchivePackConfig::new("", "", "gamedata"))
}
