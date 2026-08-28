use std::path::{Path, PathBuf};
use std::{env, fs};

use tauri::State;

use crate::core::types::TauriResult;
use crate::plugins::system::state::SystemPathsState;

/// Directory name held beside the application, or in its data directory, for everything tools write.
const OUTPUT_DIRECTORY: &str = "target";

/// Where tools write when no output directory has been configured.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "get_default_output_root"))]
#[tauri::command(rename = "get_default_output_root")]
pub async fn system_get_default_output_root(state: State<'_, SystemPathsState>) -> TauriResult<String> {
  let root: PathBuf = executable_directory()
    .filter(|directory| is_writable(directory))
    .or_else(|| state.local_data.clone())
    .ok_or_else(|| String::from("Neither the application directory nor its data directory can be written to"))?;

  Ok(root.join(OUTPUT_DIRECTORY).to_string_lossy().into_owned())
}

fn executable_directory() -> Option<PathBuf> {
  env::current_exe()
    .ok()
    .and_then(|executable| executable.parent().map(Path::to_path_buf))
}

/// Whether a directory accepts a file, tested by writing one and removing it again.
///
/// Metadata cannot answer this on Windows, where a directory under an installation root reports itself writable and
/// still refuses the write.
fn is_writable(directory: &Path) -> bool {
  let probe: PathBuf = directory.join(".xrf-write-probe");

  match fs::write(&probe, []) {
    Ok(()) => {
      let _ = fs::remove_file(&probe);

      true
    }
    Err(_) => false,
  }
}
