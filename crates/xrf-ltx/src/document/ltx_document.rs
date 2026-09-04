use crate::document::{LtxCheck, LtxItem};

/// One LTX file as it was written, before any rule is applied to it.
///
/// The single parse result in this crate. A resolved [`crate::Ltx`] is lowered from it, canonical formatting is
/// rendered from it, and an include list is read off it, so a command that verifies and reformats parses once instead
/// of twice.
///
/// Permissive: it records statements, never judges them. Rejecting a duplicate section or a duplicate
/// include belongs to whatever resolves the document, because those are dialect rules - DLTX makes a duplicate across
/// files legal through `![section]`. Nothing here performs I/O or knows what a VFS is.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LtxDocument {
  /// Every statement, including comments and blank lines, in the order written.
  pub(crate) items: Vec<LtxItem>,
  /// Checks the leading comment block opted out of.
  pub(crate) skipped_checks: Vec<LtxCheck>,
}

impl LtxDocument {
  /// Statements in written order.
  pub fn get_items(&self) -> &[LtxItem] {
    &self.items
  }

  /// Whether this file opted out of a conversion or verification check.
  pub fn is_check_skipped(&self, check: LtxCheck) -> bool {
    self.skipped_checks.contains(&check)
  }

  /// Files this document's `#include` statements name, in written order.
  ///
  /// Repeats are kept: whether naming one twice is an error is the resolver's call.
  pub fn list_included(&self) -> Vec<&str> {
    self
      .items
      .iter()
      .filter_map(|item| item.as_include().map(|(path, _)| path))
      .collect()
  }

  pub(crate) fn record_skipped_check(&mut self, check: LtxCheck) {
    if !self.is_check_skipped(check) {
      self.skipped_checks.push(check);
    }
  }
}
