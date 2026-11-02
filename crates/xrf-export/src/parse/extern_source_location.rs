use std::fmt::{Display, Formatter};

/// Location in the TypeScript source that declares an extern.
#[derive(Clone, Debug, PartialEq)]
pub struct ExternSourceLocation {
  pub column: usize,
  pub line: usize,
  /// Last line of the declaration, inclusive. Equal to `line` for a single line declaration.
  pub end_line: usize,
  pub path: String,
}

/// `path:line:column`, as diagnostics name a place.
impl Display for ExternSourceLocation {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "{}:{}:{}", self.path, self.line, self.column)
  }
}
