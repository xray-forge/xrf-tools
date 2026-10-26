/// How wide a description stores its three times: `time_t` as the tool that wrote the visual compiled it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OgfDescriptionTimeWidth {
  /// Four bytes a time.
  Narrow,
  /// Eight bytes a time.
  Wide,
}

impl OgfDescriptionTimeWidth {
  /// Bytes one time occupies.
  pub const fn get_size(self) -> u64 {
    match self {
      Self::Narrow => 4,
      Self::Wide => 8,
    }
  }
}
