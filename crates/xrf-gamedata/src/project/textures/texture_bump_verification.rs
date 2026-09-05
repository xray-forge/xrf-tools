use crate::Finding;

/// What one texture descriptor's bump declaration came to: which count it lands in, and the findings it produced.
///
/// A value per descriptor rather than a side effect on shared counters, because descriptors are judged in parallel and
/// added up afterwards in path order.
pub(crate) struct TextureBumpVerification {
  pub(crate) verdict: TextureBumpVerdict,
  pub(crate) findings: Vec<Finding>,
}

/// Which count a descriptor belongs in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextureBumpVerdict {
  /// Nothing to bind, and nothing wrong with saying so.
  Undeclared,
  /// The engine binds a pair for this descriptor, each half either the declared file or a substitute.
  Bound {
    is_bump_missing: bool,
    is_companion_missing: bool,
  },
  /// The descriptor asks for a bump the engine never reads.
  InvalidDeclaration,
}

impl TextureBumpVerification {
  pub(crate) fn undeclared() -> Self {
    Self::of(TextureBumpVerdict::Undeclared)
  }

  pub(crate) fn bound(is_bump_missing: bool, is_companion_missing: bool) -> Self {
    Self::of(TextureBumpVerdict::Bound {
      is_bump_missing,
      is_companion_missing,
    })
  }

  pub(crate) fn invalid() -> Self {
    Self::of(TextureBumpVerdict::InvalidDeclaration)
  }

  pub(crate) fn with_finding(mut self, finding: Finding) -> Self {
    self.findings.push(finding);
    self
  }

  fn of(verdict: TextureBumpVerdict) -> Self {
    Self {
      verdict,
      findings: Vec::new(),
    }
  }
}
