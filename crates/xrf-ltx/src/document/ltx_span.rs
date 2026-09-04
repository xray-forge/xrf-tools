/// Where a statement was written, one-based, as diagnostics quote it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LtxSpan {
  pub line: usize,
  pub column: usize,
}

impl LtxSpan {
  pub(crate) fn new(line: usize, column: usize) -> Self {
    Self { column, line }
  }
}
