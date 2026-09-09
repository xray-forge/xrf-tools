use xrf_pack::ArchivePatchConfig;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Write the comparison scope and header of a configuration out as a patching configuration file.
///
/// Only what such a file can carry is written, so a round trip through import returns what was exported. What is
/// compared, where it is published and under what name belong to the run rather than to the file.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "export_patch_config"))]
#[tauri::command(rename = "export_patch_config")]
pub async fn archives_export_patch_config(path: &str, config: ArchivePatchConfig) -> TauriResult<()> {
  log::info!("Exporting patch config: {path}");

  config.write_config_to_path(path).map_err(error_to_string)
}
