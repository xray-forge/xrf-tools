use crate::detail::detail_slot_fate::DetailSlotFate;
use crate::light::light_plan::LightPlan;

/// Where a slot stands once planned: met already, or waiting for its shadow rays.
pub(crate) enum DetailSlotStage {
  Met(DetailSlotFate),
  Planned(Box<LightPlan>),
}
