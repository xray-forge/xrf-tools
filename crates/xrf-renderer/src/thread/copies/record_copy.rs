use crate::capture::record::record_shot::RecordShot;
use crate::contract::viewport::render_viewport_id::RenderViewportId;

/// A recording's frame copied out of a viewport's frame, on its way back.
pub(crate) struct RecordCopy {
  pub viewport: RenderViewportId,
  pub shot: RecordShot,
}
