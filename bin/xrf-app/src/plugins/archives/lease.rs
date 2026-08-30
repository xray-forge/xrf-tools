//! What an archive job holds exclusively while it runs.
//!
//! The registry never interprets a lease key, so deciding what two runs may not do at once is this domain's business.
//! Both rules below name a destination, because that is where these operations collide: two runs reading one archive
//! are harmless, and two runs writing one tree are not.

use std::path::Path;

use xrf_pack::ArchivePackConfig;

use crate::core::jobs::to_comparable_path;

/// What a pack registers itself as, and the prefix of every lease it takes.
///
/// One constant for both, because a kind that was spelled once for the registry and again inside a key would let the
/// two drift while every test still passed: the registry does not read a key, and nothing else compares them. The
/// frontend spells the same strings in `EJobKind`, which is the wire contract this side owns.
pub const PACK_JOB_KIND: &str = "archives.pack";

/// What an unpack registers itself as, and the prefix of every lease it takes.
pub const UNPACK_JOB_KIND: &str = "archives.unpack";

/// The destination a pack would publish to, as a lease key.
///
/// Both the directory and the volume basename, because a destination directory can legitimately hold several named
/// sets: packing `gamedata` and `textures` into one folder is normal, and keying on the folder alone would serialize
/// them for no reason. Keying on the name alone would let two runs write `gamedata.db0` in different folders — which
/// is fine — but also the same one twice, which is not.
///
/// The path is canonicalized where it exists and lexically absolute where it does not, because a destination is
/// commonly typed before it is created and `canonicalize` refuses a path that is not there yet. Two spellings of an
/// existing directory therefore collide as they should; two spellings of one directory that does not exist yet may
/// not, which is the narrow gap this leaves open.
pub fn to_pack_lease_key(config: &ArchivePackConfig) -> String {
  format!(
    "{PACK_JOB_KIND}:{}|{}",
    to_comparable_path(&config.destination),
    config.name.to_lowercase()
  )
}

/// The tree an unpack would write into, as a lease key.
///
/// The root alone: an unpack writes the whole archive layout beneath it, so two runs sharing a destination overlap
/// whatever they were asked to extract.
pub fn to_unpack_lease_key(destination: &Path) -> String {
  format!("{UNPACK_JOB_KIND}:{}", to_comparable_path(destination))
}

#[cfg(test)]
mod tests {
  use std::path::{Path, PathBuf};

  use xrf_pack::ArchivePackConfig;

  use super::{to_pack_lease_key, to_unpack_lease_key};

  fn config(destination: &str, name: &str) -> ArchivePackConfig {
    ArchivePackConfig::new(PathBuf::from("C:\\src"), PathBuf::from(destination), name)
  }

  #[test]
  fn two_named_sets_in_one_directory_do_not_collide() {
    // Packing several sets into one output folder is ordinary. Keying on the folder alone would serialize them for no
    // reason at all.
    assert_ne!(
      to_pack_lease_key(&config("C:\\out", "gamedata")),
      to_pack_lease_key(&config("C:\\out", "textures"))
    );
  }

  #[test]
  fn one_named_set_in_one_directory_collides_with_itself() {
    assert_eq!(
      to_pack_lease_key(&config("C:\\out", "gamedata")),
      to_pack_lease_key(&config("C:\\out", "gamedata"))
    );
  }

  #[test]
  fn the_same_destination_spelled_differently_still_collides() {
    // The case the lease exists for: a user picking the same folder twice through a file dialog can easily produce two
    // spellings, and both runs would truncate the same volumes.
    assert_eq!(
      to_pack_lease_key(&config("C:\\Out", "gamedata")),
      to_pack_lease_key(&config("c:\\out", "GameData"))
    );
  }

  #[test]
  fn packing_and_unpacking_one_path_are_different_leases() {
    // They are not the same operation and do not write the same things: a pack writes volumes into the path, an unpack
    // writes a tree into it. Sharing a key would refuse a pair that has no conflict.
    assert_ne!(
      to_pack_lease_key(&config("C:\\out", "gamedata")),
      to_unpack_lease_key(Path::new("C:\\out"))
    );
  }

  #[test]
  fn two_unpacks_into_one_tree_collide() {
    assert_eq!(
      to_unpack_lease_key(Path::new("C:\\out\\gamedata")),
      to_unpack_lease_key(Path::new("C:\\Out\\GameData"))
    );
  }
}
