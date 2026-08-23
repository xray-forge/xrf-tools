use xrf_ltx::{LtxProject, LtxProjectOptions, LtxProjectVerifyResult, LtxVerifyOptions};
use xrf_vfs::XrayWorldSpec;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::configs::ltx_world::open_ltx_project;

/// Verify the LTX configs a world exposes.
///
/// Read-only, so it goes through the world and covers archived configs too. `xrf-ltx` draws the same
/// line: its read-only check reads through the VFS where its rewrite refuses archived winners.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "verify_directory"))]
#[tauri::command(rename = "verify_directory")]
pub async fn configs_verify_directory(
  world: XrayWorldSpec,
  prefix: Option<String>,
) -> TauriResult<LtxProjectVerifyResult> {
  log::info!("Verifying ltx configs in {}", world.describe());

  let project: LtxProject = open_ltx_project(
    &world,
    prefix.as_deref(),
    LtxProjectOptions {
      is_with_schemes_check: true,
      // todo: Probably should be provided as parameter.
      is_strict_check: false,
    },
  )
  .map_err(error_to_string)?;

  project
    .verify_entries_opt(LtxVerifyOptions::default())
    .map_err(error_to_string)
}
