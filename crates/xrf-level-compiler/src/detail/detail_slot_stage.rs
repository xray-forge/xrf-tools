use crate::detail::detail_slot_fate::DetailSlotFate;
use crate::detail::detail_slot_plan::DetailSlotPlan;

/// Where a slot stands once planned: met already, or waiting for its shadow rays.
pub(crate) enum DetailSlotStage {
  Met(DetailSlotFate),
  Planned(DetailSlotPlan),
}
