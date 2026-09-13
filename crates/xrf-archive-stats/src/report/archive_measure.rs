use serde::Serialize;

/// How many entries something holds, and what they weigh.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveMeasure {
  /// Entries counted. Directory entries are never among them; only files hold bytes.
  pub files: u64,
  /// Unpacked bytes those entries hold.
  pub size_real: u64,
}

impl ArchiveMeasure {
  /// Adds one entry of `size_real` bytes.
  pub fn add(&mut self, size_real: u64) {
    self.files += 1;
    self.size_real += size_real;
  }

  /// Whether nothing was counted, which a consumer reads as a row worth omitting.
  pub fn is_empty(&self) -> bool {
    self.files == 0
  }
}
