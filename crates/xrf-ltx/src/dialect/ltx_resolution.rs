use crate::dialect::ltx_field_origin::LtxFieldOrigin;
use crate::dialect::ltx_provenance::LtxProvenance;
use crate::dialect::ltx_resolution_diagnostic::LtxResolutionDiagnostic;
use crate::ltx::Ltx;

/// What resolving one root answers with.
///
/// Carries more than the resolved document because a caller may have to account for a value: in a modded install the
/// only way to explain one is to name the file that won it, and in any tree a field a section never wrote is worth
/// distinguishing from one it did.
#[derive(Debug, Default)]
pub struct LtxResolution {
  pub ltx: Ltx,
  /// Where each resolved field came from, when the resolution was asked for it.
  ///
  /// Empty unless [`crate::LtxResolveRequest::with_provenance`] was passed, and empty is not a claim that every field
  /// was declared where it sits - it is the absence of a record. See [`LtxProvenance`].
  pub provenance: LtxProvenance,
  /// What the dialect found worth saying that is not a failure.
  pub diagnostics: Vec<LtxResolutionDiagnostic>,
}

impl LtxResolution {
  /// A resolution with nothing recorded beside it, which is what an unasked resolve answers.
  pub fn new_plain(ltx: Ltx) -> Self {
    Self {
      diagnostics: Vec::new(),
      ltx,
      provenance: LtxProvenance::default(),
    }
  }

  /// Where one field came from, when this resolution recorded it.
  pub fn get_origin(&self, section: &str, key: &str) -> Option<&LtxFieldOrigin> {
    self.provenance.get(section, key)
  }
}
