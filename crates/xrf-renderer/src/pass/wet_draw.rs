use crate::pass::wet_stage::WetStage;

/// One stage of the wet surfaces drawn, the engine's look or the enhanced one.
#[derive(Clone, Copy, Debug)]
pub struct WetDraw {
  pub stage: WetStage,
  pub is_enhanced: bool,
}
