use xrf_pack::ArchivePackConfig;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;

/// Read a packing configuration file over the configuration the caller holds.
///
/// The codec is chosen from the path's extension, so one command reads an `ltx` and a `json` alike.
///
/// Layers rather than replaces, matching how the command line applies `--config`: a configuration file
/// carries selection rules and a header, never the source, destination, name, mode, or volume size, so
/// those stay as the caller had them.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "import_pack_config"))]
#[tauri::command(rename = "import_pack_config")]
pub async fn archives_import_pack_config(path: &str, config: ArchivePackConfig) -> TauriResult<ArchivePackConfig> {
  log::info!("Importing pack config: {path}");

  config.with_config_file(path).map_err(error_to_string)
}
