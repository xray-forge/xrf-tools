use std::fs::File;
use std::io::Result as IoResult;

/// Reads exactly `buffer.len()` bytes from `offset` without moving the file's own cursor.
///
/// # Errors
///
/// Returns an IO error when the read fails, and `UnexpectedEof` when the file ends before `buffer` is filled.
#[cfg(unix)]
pub fn read_exact_at(file: &File, buffer: &mut [u8], offset: u64) -> IoResult<()> {
  use std::os::unix::fs::FileExt;

  file.read_exact_at(buffer, offset)
}

#[cfg(windows)]
pub fn read_exact_at(file: &File, buffer: &mut [u8], offset: u64) -> IoResult<()> {
  use std::io::{Error as IoError, ErrorKind};
  use std::os::windows::fs::FileExt;

  let mut read: usize = 0;

  while read < buffer.len() {
    match file.seek_read(&mut buffer[read..], offset + read as u64)? {
      0 => {
        return Err(IoError::new(
          ErrorKind::UnexpectedEof,
          "file ended before the requested bytes",
        ));
      }
      count => read += count,
    }
  }

  Ok(())
}
