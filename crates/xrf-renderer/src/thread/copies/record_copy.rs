use crate::capture::record::record_shot::RecordShot;
use crate::contract::viewport::render_viewport_id::RenderViewportId;

/// A recording frame copied out of a viewport, awaiting readback.
pub(crate) struct RecordCopy {
  pub viewport: RenderViewportId,
  pub shot: RecordShot,
}
