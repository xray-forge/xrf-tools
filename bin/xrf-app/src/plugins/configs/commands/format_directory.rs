use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};
use xrf_vfs::XrayWorldSpec;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::configs::ltx_world::open_ltx_project;

/// Rewrite the LTX configs a world exposes.
///
/// Writing needs a file, so this refuses a project holding archived winners — the refusal comes from
/// `xrf-ltx` itself. Formatting an installation is therefore a legitimate refusal, not a gap.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "format_directory"))]
#[tauri::command(rename = "format_directory")]
pub async fn configs_format_directory(
  world: XrayWorldSpec,
  prefix: Option<String>,
) -> TauriResult<LtxProjectFormatResult> {
  log::info!("Formatting ltx configs in {}", world.describe());

  let project: LtxProject = open_ltx_project(&world, prefix.as_deref(), Default::default()).map_err(error_to_string)?;

  project
    .format_all_files_opt(LtxFormatOptions::default())
    .map_err(error_to_string)
}
