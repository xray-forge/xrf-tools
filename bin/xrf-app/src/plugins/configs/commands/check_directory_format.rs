use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};
use xrf_vfs::XrayWorldSpec;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::configs::ltx_world::open_ltx_project;

/// Report which LTX configs a world exposes are misformatted.
///
/// Read-only, so an archived config is checked like any other.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "check_directory_format"))]
#[tauri::command(rename = "check_directory_format")]
pub async fn configs_check_directory_format(
  world: XrayWorldSpec,
  prefix: Option<String>,
) -> TauriResult<LtxProjectFormatResult> {
  log::info!("Checking ltx format in {}", world.describe());

  let project: LtxProject = open_ltx_project(&world, prefix.as_deref(), Default::default()).map_err(error_to_string)?;

  project
    .check_format_all_files_opt(LtxFormatOptions::default())
    .map_err(error_to_string)
}
