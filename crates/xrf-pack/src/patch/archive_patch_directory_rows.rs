use std::collections::BTreeSet;

/// Directory rows for the ancestors of every carried entry.
///
/// A volume records the directories it contains so the engine can list a tree it has never seen loose, and a patch is
/// a volume like any other. Derived from the carried set rather than from either world: the patch holds what it holds,
/// and listing directories none of its entries live in would describe a tree it does not have.
///
/// Sorted and unique by construction. The walk up each name stops at the first ancestor already recorded, because
/// everything above that one was recorded when it was.
pub(crate) fn to_patch_directory_rows(carried: &[&str]) -> Vec<String> {
  let mut directories: BTreeSet<String> = BTreeSet::new();

  for name in carried {
    let mut ancestor: &str = name;

    while let Some((parent, _)) = ancestor.rsplit_once('\\') {
      if !directories.insert(parent.to_owned()) {
        break;
      }

      ancestor = parent;
    }
  }

  directories.into_iter().collect()
}

#[cfg(test)]
mod tests {
  use super::to_patch_directory_rows;

  #[test]
  fn every_ancestor_of_a_carried_entry_is_listed_once() {
    assert_eq!(
      to_patch_directory_rows(&["configs\\weapons\\ak74.ltx", "configs\\system.ltx"]),
      ["configs", "configs\\weapons"]
    );
  }

  #[test]
  fn an_entry_at_the_root_contributes_no_directory() {
    assert!(to_patch_directory_rows(&["system.ltx"]).is_empty());
  }
}
