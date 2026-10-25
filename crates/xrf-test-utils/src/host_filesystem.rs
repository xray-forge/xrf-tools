use std::ffi::OsString;
use std::io::Result as IoResult;
use std::path::Path;

/// What the host's filesystem accepts, for tests that pin containment and non-text path rules: links it may refuse to
/// create, and a file name it holds but `Path::to_str` refuses.
pub struct HostFilesystem;

impl HostFilesystem {
  /// Points `link` at an existing directory.
  ///
  /// A Windows symlink needs developer mode or the create-symlink privilege, which an ordinary desktop or CI account
  /// does not hold; a junction needs neither and redirects the same traversal, so it is what an unprivileged Windows
  /// run actually tests against.
  pub fn link_directory(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    let result: IoResult<()> = std::os::unix::fs::symlink(target, link);
    #[cfg(windows)]
    let result: IoResult<()> =
      std::os::windows::fs::symlink_dir(target, link).or_else(|_| Self::link_junction(target, link));

    Self::report_link(result, link)
  }

  /// Points `link` at an existing file.
  ///
  /// No junction stands in for this one, so a Windows account without the privilege skips rather than passing a check
  /// it never made.
  pub fn link_file(target: &Path, link: &Path) -> bool {
    #[cfg(unix)]
    let result: IoResult<()> = std::os::unix::fs::symlink(target, link);
    #[cfg(windows)]
    let result: IoResult<()> = std::os::windows::fs::symlink_file(target, link);

    Self::report_link(result, link)
  }

  /// A path component the host accepts and `to_str` refuses.
  ///
  /// A Linux filename is bytes, not text, and a mod tree named in CP1251 is exactly that - so `Path::to_str` answers
  /// `None` for the whole path and any rule read off that answer silently stops applying. The Windows form is an
  /// unpaired surrogate, which `PathBuf` holds as WTF-8 and `to_str` refuses the same way, so a rule stating that only
  /// a file name has to be text can be pinned on both platforms rather than only where such trees actually live.
  ///
  /// Not a name every filesystem will create: NTFS refuses a lone surrogate, so a test that writes the file gates
  /// itself on `cfg(unix)`. A test about paths alone needs no filesystem.
  pub fn non_unicode_file_name() -> OsString {
    #[cfg(unix)]
    {
      use std::os::unix::ffi::OsStringExt;

      OsString::from_vec(vec![0xD0, 0x9C, 0xFF, 0xD4])
    }

    #[cfg(windows)]
    {
      use std::os::windows::ffi::OsStringExt;

      OsString::from_wide(&[0x041C, 0xD83D, 0x043E])
    }
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
}
