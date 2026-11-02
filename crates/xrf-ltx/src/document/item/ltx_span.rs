/// Where a statement was written, one-based, as diagnostics quote it.
///
/// `u32` rather than `usize`: this sits inside every [`crate::document::LtxItem`], of which a full Anomaly config tree
/// holds hundreds of thousands, and no config has four billion lines. Callers widen at the diagnostic, which is the
/// one place the numbers are read.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LtxSpan {
  pub line: u32,
  pub column: u32,
}

impl LtxSpan {
  pub(crate) fn new(line: u32, column: u32) -> Self {
    Self { column, line }
  }

  /// The line as a diagnostic takes it.
  pub fn get_line(self) -> usize {
    self.line as usize
  }

  /// The column as a diagnostic takes it.
  pub fn get_column(self) -> usize {
    self.column as usize
  }
}
