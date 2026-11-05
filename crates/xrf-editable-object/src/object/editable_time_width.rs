use xrf_error::{XrfError, XrfResult};

/// How wide the description's time stamps are: the SDK writes its `time_t` raw, and the editors were built both with
/// the 32-bit one and with the 64-bit one later compilers default to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditableTimeWidth {
  Seconds32,
  Seconds64,
}

impl EditableTimeWidth {
  /// Bytes one time stamp takes.
  pub const fn get_size(self) -> usize {
    match self {
      Self::Seconds32 => 4,
      Self::Seconds64 => 8,
    }
  }

  /// The width that lays `tail` out as a time stamp, a null-terminated string and a second time stamp of the same
  /// width, which is what follows the creator's name.
  ///
  /// # Errors
  ///
  /// Returns an error when neither width, or both, account for every byte.
  pub fn detect(tail: &[u8]) -> XrfResult<Self> {
    let fitting: Vec<Self> = [Self::Seconds32, Self::Seconds64]
      .into_iter()
      .filter(|width| width.fits(tail))
      .collect();

    match fitting.as_slice() {
      [width] => Ok(*width),
      _ => Err(XrfError::new_parsing_error(format!(
        "Description of {} bytes after its creator is not two 32-bit or two 64-bit time stamps around a name",
        tail.len()
      ))),
    }
  }

  /// Whether the modifier's name ends exactly one time stamp before the end, with no terminator inside it.
  fn fits(self, tail: &[u8]) -> bool {
    let size: usize = self.get_size();

    tail.len() > 2 * size && tail[tail.len() - size - 1] == 0 && !tail[size..tail.len() - size - 1].contains(&0)
  }
}
