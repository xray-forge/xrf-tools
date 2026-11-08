use std::ffi::OsString;
use std::io::Result as IoResult;
use std::path::Path;

use crate::test_skip::TestSkip;

/// What the host's filesystem accepts, for tests that pin containment, non-text path and unreadable-tree rules: links it
/// may refuse to create, a file name it holds but `Path::to_str` refuses, and a directory it can be told not to list.
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

  /// Make `directory` unlistable for the running account, answering whether the denial took effect.
  ///
  /// The only portable way to make a walk fail. No account is bound by its own denial on every host, though: `root`
  /// ignores a zero mode outright. A run that cannot deny says so and skips, rather than passing a check it never made.
  pub fn deny_listing(directory: &Path) -> bool {
    Self::set_listing_denied(directory, true);

    if std::fs::read_dir(directory).is_ok() {
      TestSkip::report(format_args!(
        "'{}' stays readable for the account running this",
        directory.display()
      ));

      Self::allow_listing(directory);

      return false;
    }

    true
  }

  /// Give back what [`Self::deny_listing`] took, so the scratch tree can be removed again.
  pub fn allow_listing(directory: &Path) {
    Self::set_listing_denied(directory, false);
  }

  #[cfg(unix)]
  fn set_listing_denied(directory: &Path, is_denied: bool) {
    use std::os::unix::fs::PermissionsExt;

    let mode: u32 = if is_denied { 0o000 } else { 0o755 };

    std::fs::set_permissions(directory, std::fs::Permissions::from_mode(mode)).expect("directory mode");
  }

  /// Windows has no mode bits, so a deny entry for the running account is what stops this process listing the directory.
  ///
  /// A failure is left to the caller's own readability check, because a filesystem that carries no access list refuses
  /// the command and stays readable, which is exactly the case that should skip.
  #[cfg(windows)]
  fn set_listing_denied(directory: &Path, is_denied: bool) {
    use std::process::{Command, Stdio};

    let user: String = std::env::var("USERNAME").expect("the running account");
    let rule: [String; 2] = if is_denied {
      [String::from("/deny"), format!("{user}:(RD)")]
    } else {
      [String::from("/remove:d"), user]
    };

    let _ = Command::new("icacls")
      .arg(directory)
      .args(rule)
      .stdout(Stdio::null())
      .stderr(Stdio::null())
      .status();
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
        TestSkip::report(format_args!(
          "this host cannot create the link '{}': {error}",
          link.display()
        ));

        false
      }
    }
  }
}
