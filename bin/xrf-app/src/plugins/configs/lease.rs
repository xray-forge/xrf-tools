//! What a configs job registers itself as, and what a rewriting one holds exclusively.
//!
//! Only formatting takes a lease. Verification reads, and two readers of one project have nothing to collide over.

use xrf_vfs::{XrayRoot, XrayRoots};

use crate::core::jobs::to_comparable_path;

/// What a configs verification registers itself as.
///
/// The frontend spells the same string in `EJobKind`, which is the wire contract this side owns.
pub const VERIFY_JOB_KIND: &str = "configs.verify";

/// What a configs formatting run registers itself as, and the prefix of every lease it takes.
pub const FORMAT_JOB_KIND: &str = "configs.format";

/// What a configs formatting check registers itself as.
///
/// Separate from the rewrite it reports on: one answers a question and the other changes the files, so they are
/// different work to watch, to attribute, and to decide about. Only the rewrite takes a lease.
pub const CHECK_FORMAT_JOB_KIND: &str = "configs.check-format";

/// The configs a formatting run would rewrite, as a lease key.
///
/// Every root in search order plus the prefix, because that pair is what decides which files the run resolves as
/// writable winners. Keying on the first root alone would let two runs over overlapping sets rewrite the same file.
///
/// Ordered rather than sorted: the order is what picks a winner between two roots holding one logical path, so two
/// root sets that differ only in order genuinely address different files.
pub fn to_format_lease_key(roots: &XrayRoots, prefix: Option<&str>) -> String {
  let paths: String = roots
    .roots
    .iter()
    .map(|root: &XrayRoot| to_comparable_path(&root.path))
    .collect::<Vec<String>>()
    .join(";");

  format!(
    "{FORMAT_JOB_KIND}:{paths}|{}",
    prefix.unwrap_or_default().to_lowercase()
  )
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

  use super::to_format_lease_key;

  fn roots(paths: &[&str]) -> XrayRoots {
    XrayRoots::new(
      paths
        .iter()
        .map(|path| XrayRoot::new(PathBuf::from(*path), XrayMountMode::Auto)),
    )
  }

  #[test]
  fn one_project_collides_with_itself() {
    assert_eq!(
      to_format_lease_key(&roots(&["C:\\gamedata"]), Some("configs")),
      to_format_lease_key(&roots(&["c:\\GameData"]), Some("CONFIGS"))
    );
  }

  #[test]
  fn two_subtrees_of_one_project_do_not_collide() {
    // Formatting `configs\\scripts` while another run formats `configs\\ui` touches no shared file, and serializing
    // them would refuse a pair that has no conflict.
    assert_ne!(
      to_format_lease_key(&roots(&["C:\\gamedata"]), Some("configs\\scripts")),
      to_format_lease_key(&roots(&["C:\\gamedata"]), Some("configs\\ui"))
    );
  }

  #[test]
  fn the_whole_project_is_its_own_key() {
    assert_ne!(
      to_format_lease_key(&roots(&["C:\\gamedata"]), None),
      to_format_lease_key(&roots(&["C:\\gamedata"]), Some("configs"))
    );
  }

  #[test]
  fn root_order_is_part_of_the_identity() {
    // The order picks the winner between two roots holding one logical path, so a reordered set addresses different
    // files and is not the same run.
    assert_ne!(
      to_format_lease_key(&roots(&["C:\\mod", "C:\\game"]), None),
      to_format_lease_key(&roots(&["C:\\game", "C:\\mod"]), None)
    );
  }
}
