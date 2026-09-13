//! Showing a path in whatever file manager the desktop provides.
//!
//! One function per platform rather than one function with branches, because the three have nothing in common beyond
//! their signature: each spawns a different program with a different convention for selecting an item.

use std::io;
use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
use xrf_utils::format_path;

/// Open the file manager at a directory, or at a file with the file selected.
#[cfg(target_os = "windows")]
pub fn reveal_path(target: &Path) -> io::Result<()> {
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
pub fn reveal_path(target: &Path) -> io::Result<()> {
  Command::new("open").arg("-R").arg(target).spawn()?;

  Ok(())
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn reveal_path(target: &Path) -> io::Result<()> {
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
