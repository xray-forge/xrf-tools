use serde::Serialize;

use crate::inventory::{LtxInventoryFile, LtxInventoryRole};

/// Every config a project holds, and what each one is to the project.
///
/// The tree a person navigates and the list `ltx list` prints are the same record: one place decides what an entry
/// point is, so a command and a viewer can never disagree about which files stand on their own.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LtxInventory {
  /// Configs sorted by engine identity, which is the order the project itself assembles them in.
  pub files: Vec<LtxInventoryFile>,
}

impl LtxInventory {
  /// One config's entry, by engine identity.
  pub fn find(&self, path: &str) -> Option<&LtxInventoryFile> {
    self.files.iter().find(|file| file.path == path)
  }

  /// Every entry point whose resolution reaches one config, in project order.
  ///
  /// Walks the include graph upward rather than storing the answer per file: a config included by twenty others has
  /// one or two entry points above it, and storing the closure for every file of an Anomaly tree would be a table
  /// larger than the inventory itself. An entry point answers itself.
  pub fn list_entry_points_of(&self, path: &str) -> Vec<String> {
    let mut reached: Vec<String> = Vec::new();
    let mut pending: Vec<String> = vec![String::from(path)];
    let mut seen: Vec<String> = vec![String::from(path)];

    while let Some(current) = pending.pop() {
      let Some(file) = self.find(&current) else {
        continue;
      };

      if matches!(file.role, LtxInventoryRole::Attachment) {
        continue;
      }

      if file.included_by.is_empty() {
        reached.push(current);

        continue;
      }

      for includer in &file.included_by {
        if seen.iter().any(|held| held == includer) {
          continue;
        }

        seen.push(includer.clone());
        pending.push(includer.clone());
      }
    }

    // Project order, so a file reached by two entry points names them the way the project lists them.
    reached.sort();

    reached
  }
}
