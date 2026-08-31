//! What a gamedata run calls the work it is doing.

/// Phase a verification reports while it works through the checks it was asked for.
///
/// Named for the units it counts rather than for the operation, because the checks nested inside it are verifications
/// too: a stack reading `verify / verify` would say nothing about which level was which.
pub const GAMEDATA_PHASE_CHECKS: &str = "checks";
