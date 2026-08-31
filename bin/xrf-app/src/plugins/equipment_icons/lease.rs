//! What an equipment-icon job registers itself as, and what it holds exclusively.

use std::path::Path;

use crate::core::jobs::to_comparable_path;

/// What packing an equipment sprite sheet registers itself as, and the prefix of every lease it takes.
///
/// The frontend spells the same string in `EJobKind`, which is the wire contract this side owns.
pub const PACK_SPRITE_JOB_KIND: &str = "equipment-icons.pack";

/// The sheet a run would write, as a lease key.
///
/// The output file itself rather than its directory: a sprite sheet is one image written once, so two runs collide
/// only where they would write the same file, and unrelated sheets in one folder have no reason to queue.
pub fn to_pack_sprite_lease_key(output_path: &Path) -> String {
  format!("{PACK_SPRITE_JOB_KIND}:{}", to_comparable_path(output_path))
}
