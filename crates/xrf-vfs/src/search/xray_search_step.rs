use xrf_error::XrfResult;

use crate::{XrayLookupScope, XrayMount, XraySkippedMount};

/// One place a search looks, and the name a report calls it by: what Source's file system calls a path ID, a named
/// group of mounts searched together.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XraySearchStep {
  label: String,
  scope: XrayLookupScope,
  skipped: Vec<XraySkippedMount>,
}

impl XraySearchStep {
  pub fn new(label: impl Into<String>, scope: XrayLookupScope) -> Self {
    Self::planned(label, scope, Vec::new())
  }

  /// A step as [`crate::XraySearchPlan::mount_into`] produced it, carrying what its own plan could not open.
  pub(crate) fn planned(label: impl Into<String>, scope: XrayLookupScope, skipped: Vec<XraySkippedMount>) -> Self {
    Self {
      label: label.into(),
      scope,
      skipped,
    }
  }

  /// This step searching the logical subtree `prefix` alone, its own prefix replaced.
  pub(crate) fn with_prefix(self, prefix: &str) -> XrfResult<Self> {
    Ok(Self {
      scope: self.scope.with_prefix(prefix)?,
      ..self
    })
  }

  pub fn get_label(&self) -> &str {
    &self.label
  }

  pub fn get_scope(&self) -> &XrayLookupScope {
    &self.scope
  }

  /// Sources this step's plan named that could not be opened, so this step answers for none of their entries.
  pub fn get_skipped(&self) -> &[XraySkippedMount] {
    &self.skipped
  }
}

/// One source a search reaches, and the step that reaches it.
#[derive(Clone, Copy, Debug)]
pub struct XraySearchedSource<'a> {
  /// Label of the search step that reaches this source.
  pub step: &'a str,
  /// The mounted source itself, which answers for what it holds and how the plan described it.
  pub mount: &'a XrayMount,
}
