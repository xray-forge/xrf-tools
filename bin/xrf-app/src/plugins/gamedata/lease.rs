//! What a gamedata job registers itself as.
//!
//! No lease: verification only reads, and two readers of one project have nothing to collide over. A run that wrote
//! would want one keyed on the root it wrote to.

/// What a gamedata verification registers itself as.
///
/// The frontend spells the same string in `EJobKind`, which is the wire contract this side owns.
pub const VERIFY_JOB_KIND: &str = "gamedata.verify";
