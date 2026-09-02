use std::fs::File;
use std::io::Result as IoResult;

/// Reads exactly `buffer.len()` bytes from `offset` without moving the file's own cursor.
///
/// Positional reads are what let one handle serve several readers at once: a cursor is state they would race for,
/// while both platform calls below take `&File` and carry the position in the call. A reader that shares a handle
/// across threads needs neither a lock nor a handle per thread.
///
/// The two platforms are not symmetrical, which is the trap this exists to hold in one place: Unix provides the
/// exact-read loop, Windows provides only the short-read primitive.
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
