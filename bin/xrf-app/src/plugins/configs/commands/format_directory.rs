use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};
use xrf_vfs::XrayRoots;

use crate::core::error::error_to_string;
use crate::core::types::TauriResult;
use crate::plugins::configs::ltx_roots::open_ltx_project;

/// Rewrite the LTX configs roots exposes.
///
/// Writing needs a file, so this refuses a project holding archived winners — the refusal comes from
/// `xrf-ltx` itself. Formatting an installation is therefore a legitimate refusal, not a gap.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "format_directory"))]
#[tauri::command(rename = "format_directory")]
pub async fn configs_format_directory(roots: XrayRoots, prefix: Option<String>) -> TauriResult<LtxProjectFormatResult> {
  log::info!("Formatting ltx configs in {}", roots.describe());

  let project: LtxProject = open_ltx_project(&roots, prefix.as_deref(), Default::default()).map_err(error_to_string)?;

  project
    .format_all_files_opt(LtxFormatOptions::default())
    .map_err(error_to_string)
}
