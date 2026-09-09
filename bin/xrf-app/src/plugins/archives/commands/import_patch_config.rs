use xrf_pack::ArchivePatchConfig;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Read a patching configuration file over the configuration the caller holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "import_patch_config"))]
#[tauri::command(rename = "import_patch_config")]
pub async fn archives_import_patch_config(path: &str, config: ArchivePatchConfig) -> TauriResult<ArchivePatchConfig> {
  log::info!("Importing patch config: {path}");

  config.with_config_file(path).map_err(error_to_string)
}
