use xrf_pack::ArchivePatchConfig;

use crate::core::types::TauriResult;

/// Returns the format defaults with empty paths and the volume name `patch`.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "default_patch_config"))]
#[tauri::command(rename = "default_patch_config")]
pub async fn archives_default_patch_config() -> TauriResult<ArchivePatchConfig> {
  Ok(ArchivePatchConfig::new("", "", "", "patch"))
}
