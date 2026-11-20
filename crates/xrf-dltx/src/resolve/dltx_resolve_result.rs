use std::collections::BTreeMap;
use std::sync::Arc;

use crate::resolve::dltx_diagnostic::DltxDiagnostic;
use crate::resolve::dltx_provenance::DltxProvenance;

/// What one resolved config tree holds, and how each value got there.
#[derive(Debug, Default)]
pub(crate) struct DltxResolveResult {
  /// Sections by lowercased name, each field by key, in the engine's emitted order.
  pub sections: BTreeMap<Arc<str>, BTreeMap<Arc<str>, Arc<str>>>,
  pub provenance: DltxProvenance,
  pub diagnostics: Vec<DltxDiagnostic>,
}

impl DltxResolveResult {
  /// One field's resolved value.
  #[cfg(test)]
  pub(crate) fn get(&self, section: &str, key: &str) -> Option<&str> {
    self
      .sections
      .get(section)
      .and_then(|fields| fields.get(key))
      .map(|value| &**value)
  }

  /// Section names, sorted.
  #[cfg(test)]
  pub(crate) fn list_sections(&self) -> Vec<&str> {
    self.sections.keys().map(|section| &**section).collect()
  }
}
