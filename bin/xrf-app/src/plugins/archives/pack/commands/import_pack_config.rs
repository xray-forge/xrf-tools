use xrf_pack::ArchivePackConfig;
use xrf_utils::error_to_string;

use crate::core::types::TauriResult;

/// Read a packing configuration file over the configuration the caller holds.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "import_pack_config"))]
#[tauri::command(rename = "import_pack_config")]
pub async fn archives_import_pack_config(path: &str, config: ArchivePackConfig) -> TauriResult<ArchivePackConfig> {
  log::info!("Importing pack config: {path}");

  config.with_config_file(path).map_err(error_to_string)
}
