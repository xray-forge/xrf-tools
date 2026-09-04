use std::collections::BTreeMap;

use crate::resolve::dltx_diagnostic::DltxDiagnostic;
use crate::resolve::dltx_provenance::DltxProvenance;

/// What one resolved config tree holds, and how each value got there.
#[derive(Debug, Default)]
pub struct DltxResolveResult {
  /// Sections by lowercased name, each field by key, in the engine's emitted order.
  pub sections: BTreeMap<String, BTreeMap<String, String>>,
  pub provenance: DltxProvenance,
  pub diagnostics: Vec<DltxDiagnostic>,
}

impl DltxResolveResult {
  /// One field's resolved value.
  pub fn get(&self, section: &str, key: &str) -> Option<&str> {
    self
      .sections
      .get(section)
      .and_then(|fields| fields.get(key))
      .map(String::as_str)
  }

  /// Section names, sorted.
  pub fn list_sections(&self) -> Vec<&str> {
    self.sections.keys().map(String::as_str).collect()
  }
}
