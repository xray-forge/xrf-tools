use std::io;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
use xrf_utils::format_path;

use crate::core::types::TauriResult;

/// Show a path in the desktop's own file manager.
///
/// This exists instead of the shell plugin's `open` because that command validates what it is handed
/// against a regex which only matches `http`, `mailto` and `tel`, so a filesystem path is always
/// rejected. Widening that scope would allow opening any file with its default handler, executables
/// included, while this only ever hands a path to the file manager.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "reveal_path"))]
#[tauri::command(rename = "reveal_path")]
pub async fn system_reveal_path(path: &str) -> TauriResult<()> {
  let target: &Path = Path::new(path);

  if !target.exists() {
    return Err(format!("Cannot show a path that does not exist: {path}"));
  }

  log::info!("Revealing path: {}", path);

  reveal(target).map_err(|error| format!("Could not show {path}: {error}"))
}

/// Open the file manager at a directory, or at a file with the file selected.
#[cfg(target_os = "windows")]
fn reveal(target: &Path) -> io::Result<()> {
  use std::os::windows::process::CommandExt;

  let mut command: Command = Command::new("explorer");

  // Written as one raw argument because `explorer` parses `/select,<path>` itself: passed as a normal
  // argument the quoting lands around the whole token and it opens the user's documents instead.
  if target.is_dir() {
    command.raw_arg(format!("\"{}\"", format_path(target)));
  } else {
    command.raw_arg(format!("/select,\"{}\"", format_path(target)));
  }

  // Spawned rather than waited on: `explorer` exits with a failing status even when it opened the
  // window it was asked for, so its result carries no information.
  command.spawn()?;

  Ok(())
}

#[cfg(target_os = "macos")]
fn reveal(target: &Path) -> io::Result<()> {
  Command::new("open").arg("-R").arg(target).spawn()?;

  Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn reveal(target: &Path) -> io::Result<()> {
  // Selecting the item needs the file manager's own dbus interface, which not every desktop provides,
  // so the containing directory is opened and the file is merely visible in it.
  let directory: &Path = if target.is_dir() {
    target
  } else {
    target.parent().unwrap_or(target)
  };

  Command::new("xdg-open").arg(directory).spawn()?;

  Ok(())
}
