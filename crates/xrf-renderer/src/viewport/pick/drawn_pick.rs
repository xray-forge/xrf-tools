use crate::viewport::pick::pending_pick::PendingPick;
use crate::viewport::pick::pick_unprojection::PickUnprojection;

/// A pick a viewport's frame draws this frame, and how its depth unprojects once it is back.
pub(crate) struct DrawnPick {
  pub pick: PendingPick,
  pub unprojection: PickUnprojection,
}
