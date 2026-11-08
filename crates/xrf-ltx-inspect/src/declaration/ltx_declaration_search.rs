use std::collections::BTreeSet;

use serde::Serialize;
use xrf_ltx::LtxSectionOperation;

use crate::declaration::{LtxDeclaration, LtxDeclarationQuery, LtxDeclarationStatement, LtxUnreadableConfig};
use crate::inventory::{LtxInventory, LtxInventoryRole};

/// What one declaration search found across a project.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxDeclarationSearch {
  /// The query, echoed back.
  pub query: LtxDeclarationQuery,
  /// Every matching statement, in the inventory's config order and then in document order.
  pub declarations: Vec<LtxDeclaration>,
  /// Configs that would not parse, and so declare nothing this search could see.
  pub unreadable: Vec<LtxUnreadableConfig>,
}

impl LtxDeclarationSearch {
  /// Entry points whose resolution can hold a section this search found headers for, sorted.
  ///
  /// A superset of the entry points that do hold it: a header reaches a resolution only through the entry points
  /// above its config, and resolving one of them is what says whether the section survived. That lets a caller resolve
  /// those few rather than every entry point of the tree. A config in `unreadable` names none, and needs to: neither it
  /// nor anything including it resolves.
  ///
  /// `None` when no superset can be named: an attachment holding a header that can create the section - a plain one,
  /// or `@` - adds it to a root nothing records, since a dialect's attachment plan drops the pairing. An attachment's
  /// `!` or `!!` header only edits a section some config already declares, so it narrows nothing away.
  pub fn list_holding_entry_points(&self, inventory: &LtxInventory) -> Option<Vec<String>> {
    let creating: [&str; 2] = [
      LtxSectionOperation::Declare.as_prefix(),
      LtxSectionOperation::SafeOverride.as_prefix(),
    ];
    let mut holding: BTreeSet<String> = BTreeSet::new();

    for declaration in &self.declarations {
      if !matches!(declaration.statement, LtxDeclarationStatement::Section { .. }) {
        continue;
      }

      let is_attachment: bool = inventory
        .find(&declaration.path)
        .is_some_and(|file| matches!(file.role, LtxInventoryRole::Attachment));

      if is_attachment {
        if creating.contains(&declaration.section_operation.as_str()) {
          return None;
        }

        continue;
      }

      holding.extend(inventory.list_entry_points_of(&declaration.path));
    }

    Some(holding.into_iter().collect())
  }
}
