use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;

/// One open log file and its path, held on Windows without delete sharing.
pub struct LogFile {
  pub path: PathBuf,
  pub file: File,
}

impl LogFile {
  /// `FILE_SHARE_READ | FILE_SHARE_WRITE`: another process reads or appends, none deletes or renames.
  #[cfg(windows)]
  const SHARE_READ_WRITE: u32 = 0x1 | 0x2;

  /// Creates the log at `path`, emptying one already there.
  ///
  /// # Errors
  ///
  /// Returns the filesystem's error where the file cannot be made.
  pub fn create(path: PathBuf) -> io::Result<Self> {
    let mut options: OpenOptions = OpenOptions::new();

    options.write(true).create(true).truncate(true);

    #[cfg(windows)]
    std::os::windows::fs::OpenOptionsExt::share_mode(&mut options, Self::SHARE_READ_WRITE);

    options.open(&path).map(|file| Self { path, file })
  }
}
