use xrf_pack::ArchivePackConfig;
use xrf_utils::error_to_string;

use crate::core::types::TauriResult;

/// Write the selection rules of a configuration out as a packing configuration file.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "export_pack_config"))]
#[tauri::command(rename = "export_pack_config")]
pub async fn archives_export_pack_config(path: &str, config: ArchivePackConfig) -> TauriResult<()> {
  log::info!("Exporting pack config: {path}");

  config.write_config_to_path(path).map_err(error_to_string)
}
