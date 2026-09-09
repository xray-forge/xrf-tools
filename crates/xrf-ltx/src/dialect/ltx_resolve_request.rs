/// What a caller wants out of resolving a root, beyond the resolved config itself.
///
/// [`Default`] is the plain resolution, so a caller that says nothing asks for nothing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LtxResolveRequest {
  /// Whether to record where each resolved field came from.
  is_with_provenance: bool,
}

impl LtxResolveRequest {
  /// The resolved config alone, which is what every check and every format wants.
  pub fn plain() -> Self {
    Self::default()
  }

  /// The resolved config with a per-field origin beside it, for a surface that has to explain a value.
  pub fn with_provenance() -> Self {
    Self {
      is_with_provenance: true,
    }
  }

  /// Whether the caller asked for per-field origins.
  pub fn is_with_provenance(self) -> bool {
    self.is_with_provenance
  }
}
