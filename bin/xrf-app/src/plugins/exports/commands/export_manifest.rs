use std::path::Path;
use std::sync::Arc;

use tauri::State;
use xrf_export::{ExportsProject, ExternFormat, write_extern_manifest};

use crate::core::error::error_to_string;
use crate::core::session::{SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::exports::state::ExportsProjectState;

#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "export_manifest"))]
#[tauri::command(rename = "export_manifest")]
pub async fn exports_export_manifest(
  session_id: SessionId,
  path: &str,
  state: State<'_, ExportsProjectState>,
) -> TauriResult<()> {
  log::info!("Exporting xr externs manifest: {path}");

  let project: Arc<SessionSnapshot<ExportsProject>> = state.require(session_id)?;
  let destination: &Path = Path::new(path);
  let format: ExternFormat = ExternFormat::from_extension(destination).map_err(error_to_string)?;

  write_extern_manifest(&project.manifest, destination, format, None).map_err(error_to_string)
}
