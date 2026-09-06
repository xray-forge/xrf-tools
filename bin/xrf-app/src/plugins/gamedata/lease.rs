//! Gamedata verification identity and exclusion group.
//!
//! The constant group allows one verification across all windows, independently of the selected root.

/// What a gamedata verification registers itself as.
///
/// The frontend spells the same string in `EJobKind`, which is the wire contract this side owns.
pub const VERIFY_JOB_KIND: &str = "gamedata.verify";
