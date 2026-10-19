//! Filesystem links a containment test writes against.

use std::io::Result as IoResult;
use std::path::Path;

/// Points `link` at an existing directory.
///
/// A Windows symlink needs developer mode or the create-symlink privilege, which an ordinary desktop or CI account
/// does not hold; a junction needs neither and redirects the same traversal, so it is what an unprivileged Windows run
/// actually tests against.
pub fn link_directory(target: &Path, link: &Path) -> bool {
  #[cfg(unix)]
  let result: IoResult<()> = std::os::unix::fs::symlink(target, link);
  #[cfg(windows)]
  let result: IoResult<()> = std::os::windows::fs::symlink_dir(target, link).or_else(|_| link_junction(target, link));

  report_link(result, link)
}

/// Points `link` at an existing file.
///
/// No junction stands in for this one, so a Windows account without the privilege skips rather than passing a check it
/// never made.
pub fn link_file(target: &Path, link: &Path) -> bool {
  #[cfg(unix)]
  let result: IoResult<()> = std::os::unix::fs::symlink(target, link);
  #[cfg(windows)]
  let result: IoResult<()> = std::os::windows::fs::symlink_file(target, link);

  report_link(result, link)
}

/// One raw argument, because `cmd` re-parses the line and both paths may hold spaces.
#[cfg(windows)]
fn link_junction(target: &Path, link: &Path) -> IoResult<()> {
  use std::io::Error as IoError;
  use std::os::windows::process::CommandExt;
  use std::process::{Command, Stdio};

  let status = Command::new("cmd")
    .raw_arg(format!("/C mklink /J \"{}\" \"{}\"", link.display(), target.display()))
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .status()?;

  if status.success() {
    Ok(())
  } else {
    Err(IoError::other(format!("mklink /J exited with {status}")))
  }
}

fn report_link(result: IoResult<()>, link: &Path) -> bool {
  match result {
    Ok(()) => true,
    Err(error) => {
      eprintln!(
        "skipping: this host cannot create the link '{}': {error}",
        link.display()
      );

      false
    }
  }
}
