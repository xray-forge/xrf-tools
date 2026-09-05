use std::collections::{BTreeMap, BTreeSet};

use crate::load::dltx_item::DltxItem;
use crate::resolve::dltx_diagnostic::DltxDiagnostic;

/// Everything one load pass recorded, before any of it is resolved.
#[derive(Debug, Default)]
pub struct DltxLoadResult {
  /// Plain sections, by lowercased name.
  pub(crate) base: BTreeMap<String, Vec<DltxItem>>,
  /// `![section]` and `@[section]` contents, by lowercased name.
  pub(crate) overrides: BTreeMap<String, Vec<DltxItem>>,
  /// `>key` and `<key` operations, by lowercased section name, in the order read.
  pub(crate) list_operations: BTreeMap<String, Vec<DltxItem>>,
  /// `!![section]` targets, applied after everything else resolves.
  pub(crate) deleted_sections: BTreeSet<String>,
  /// Parent tokens declared by plain headers. A token may carry a leading `!`, meaning removal.
  pub(crate) base_parents: BTreeMap<String, Vec<String>>,
  /// Parent tokens declared by override headers.
  pub(crate) override_parents: BTreeMap<String, Vec<String>>,
  /// `@[section]` names, which get an empty base section when nothing else declares one.
  pub(crate) safe_created: BTreeSet<String>,
  /// Which file declared each plain section, for the duplicate diagnostic.
  /// Which config declared each section, as the whole logical path rather than the bare name.
  pub(crate) section_files: BTreeMap<String, String>,
  /// What the load pass found worth reporting.
  pub(crate) diagnostics: Vec<DltxDiagnostic>,
}

impl DltxLoadResult {
  /// Gives every `@[section]` an empty base section when nothing declared one.
  ///
  /// What separates a safe override from a plain one: the section is created rather than the override being dropped,
  /// and it bypasses the duplicate refusal.
  pub(crate) fn create_safe_sections(&mut self) {
    for section in &self.safe_created {
      self.base.entry(section.clone()).or_default();
    }
  }
}
