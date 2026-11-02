use crate::capture::burst::burst_shot::BurstShot;
use crate::contract::viewport::render_viewport_id::RenderViewportId;

/// A burst's frame copied out of a viewport's frame, on its way back.
pub struct BurstCopy {
  pub viewport: RenderViewportId,
  pub shot: BurstShot,
}
