use std::fs::File;
use std::io::{Read, Result as IoResult};

/// Read whole file as string.
pub fn read_file_as_string(file: &mut File) -> IoResult<String> {
  let mut value: String = String::new();

  file.read_to_string(&mut value)?;

  Ok(value)
}
