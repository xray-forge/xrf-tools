use std::fmt::{Display, Formatter, Result as FormatterResult};

use serde::Serialize;

/// The kind of an operation that can be registered, cancelled and rediscovered: its domain's plugin name, a dot and the
/// operation (`archives.pack`).
///
/// Each domain declares its own kinds in its registry (`@jobs`), which builds them under its plugin name, so the
/// registry knows no domain and no domain can spell another's kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
pub struct JobKind(&'static str);

impl JobKind {
  /// The kind spelled `name` on the wire, which a domain's registry builds from its plugin name.
  pub const fn new(name: &'static str) -> Self {
    Self(name)
  }

  pub const fn as_str(self) -> &'static str {
    self.0
  }
}

impl Display for JobKind {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatterResult {
    formatter.write_str(self.0)
  }
}
