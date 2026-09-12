use xrf_pack::ArchivePackConfig;

use crate::core::types::TauriResult;

/// Hand back a packing configuration with nothing chosen yet.
///
/// The editor starts from this rather than from its own literals, so defaults that belong to the format
/// - the volume ceiling, the skip list, the mode - have one definition, in the packer.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "default_pack_config"))]
#[tauri::command(rename = "default_pack_config")]
pub async fn archives_default_pack_config() -> TauriResult<ArchivePackConfig> {
  Ok(ArchivePackConfig::new("", "", "gamedata"))
}
