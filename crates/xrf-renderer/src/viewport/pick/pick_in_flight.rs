use crate::viewport::pick::pending_pick::PendingPick;
use crate::viewport::pick::pick_unprojection::PickUnprojection;

/// A pick drawn and on its way back: the readback it went into, how its depth unprojects, and the frame it was drawn
/// in.
pub struct PickInFlight {
  pub pick: PendingPick,
  pub slot: usize,
  pub unprojection: PickUnprojection,
  pub frame: u64,
}
