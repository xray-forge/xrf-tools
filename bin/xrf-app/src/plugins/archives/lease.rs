//! Archive job identities, exclusion groups, and destination leases.

use std::path::Path;

use crate::core::jobs::to_comparable_path;

/// Archive packing job kind; mirrored by the frontend `EJobKind`.
pub const PACK_JOB_KIND: &str = "archives.pack";

/// Archive patch publication job kind.
pub const PATCH_JOB_KIND: &str = "archives.patch";

/// Read-only archive comparison job kind.
pub const COMPARE_JOB_KIND: &str = "archives.compare";

/// Limits archive packing and patch publication to one run at a time across windows.
pub const PUBLISH_ACTION_GROUP: &str = "archives.publish";

/// Archive unpacking job kind.
pub const UNPACK_JOB_KIND: &str = "archives.unpack";

/// Archive directory extraction job kind.
pub const EXTRACT_JOB_KIND: &str = "archives.extract";

/// Shared destination lease prefix for unpacking and directory extraction.
const DESTINATION_TREE_LEASE: &str = "archives.tree";

/// Identifies a published set by destination and case-insensitive volume name.
///
/// Shared by packing and patching. Existing paths are canonicalized; missing paths use lexical absolute paths,
/// which may leave aliases undetected.
pub fn to_published_set_lease_key(destination: &Path, name: &str) -> String {
  format!(
    "{PUBLISH_ACTION_GROUP}:{}|{}",
    to_comparable_path(destination),
    name.to_lowercase()
  )
}

/// Identifies an unpack or extraction destination, regardless of the selected archive prefix.
pub fn to_destination_tree_lease_key(destination: &Path) -> String {
  format!("{DESTINATION_TREE_LEASE}:{}", to_comparable_path(destination))
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use super::{to_destination_tree_lease_key, to_published_set_lease_key};

  fn published(destination: &str, name: &str) -> String {
    to_published_set_lease_key(&PathBuf::from(destination), name)
  }

  #[test]
  fn two_named_sets_in_one_directory_do_not_collide() {
    // Destination identity stays distinct even though the action group excludes concurrent packs.
    assert_ne!(published("C:\\out", "gamedata"), published("C:\\out", "textures"));
  }

  #[test]
  fn one_named_set_in_one_directory_collides_with_itself() {
    assert_eq!(published("C:\\out", "gamedata"), published("C:\\out", "gamedata"));
  }

  #[test]
  fn the_same_destination_spelled_differently_still_collides() {
    // The case the lease exists for: a user picking the same folder twice through a file dialog can easily produce two
    // spellings, and both runs would truncate the same volumes.
    assert_eq!(published("C:\\Out", "gamedata"), published("c:\\out", "GameData"));
  }

  #[test]
  fn publishing_a_set_and_writing_a_tree_are_different_leases() {
    // They are not the same operation and do not write the same things: a pack writes volumes into the path, an unpack
    // writes a tree into it. Sharing a key would refuse a pair that has no conflict.
    assert_ne!(
      published("C:\\out", "gamedata"),
      to_destination_tree_lease_key(Path::new("C:\\out"))
    );
  }

  #[test]
  fn an_extraction_and_an_unpack_share_one_destination_tree() {
    // Both lay the archive's layout into the root, so they overlap whatever each was asked for. The key is named after
    // the tree rather than either operation for exactly this reason.
    assert_eq!(
      to_destination_tree_lease_key(Path::new("C:\\out")),
      to_destination_tree_lease_key(Path::new("c:\\OUT"))
    );
  }

  #[test]
  fn two_unpacks_into_one_tree_collide() {
    assert_eq!(
      to_destination_tree_lease_key(Path::new("C:\\out\\gamedata")),
      to_destination_tree_lease_key(Path::new("C:\\Out\\GameData"))
    );
  }
}
