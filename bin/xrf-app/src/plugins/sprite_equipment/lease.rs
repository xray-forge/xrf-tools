//! What an equipment-icon job registers itself as, and what it holds exclusively.

use std::path::Path;

use crate::core::jobs::to_comparable_path;

/// What packing an equipment sprite sheet registers itself as, and the prefix of every lease it takes.
///
/// The frontend spells the same string in `EJobKind`, which is the wire contract this side owns.
pub const PACK_SPRITE_JOB_KIND: &str = "sprite-equipment.pack";

/// The sheet a run would write, as a lease key.
///
/// Identifies one output image. The constant action group separately excludes concurrent packs of different sheets.
pub fn to_pack_sprite_lease_key(output_path: &Path) -> String {
  format!("{PACK_SPRITE_JOB_KIND}:{}", to_comparable_path(output_path))
}
