/// A conversion or verification check that an LTX file can opt out of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LtxCheck {
  /// Resolve section inheritance before consumers read the file.
  Inheritance,
}

impl LtxCheck {
  pub(crate) fn from_skip_directive(directive: &str) -> Option<Self> {
    match directive {
      "skip-inheritance" => Some(Self::Inheritance),
      _ => None,
    }
  }
}
