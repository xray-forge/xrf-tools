/// What the console answers a line with: the lines it shows, and whether the line failed, which the bridge reports as
/// an error.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ConsoleAnswer {
  pub lines: Vec<String>,
  pub is_failed: bool,
}

impl ConsoleAnswer {
  /// A line done, answered with `lines`.
  pub fn succeed(lines: Vec<String>) -> Self {
    Self {
      lines,
      is_failed: false,
    }
  }

  /// A line refused, for `reason`.
  pub fn fail(reason: impl Into<String>) -> Self {
    Self {
      lines: vec![reason.into()],
      is_failed: true,
    }
  }
}
