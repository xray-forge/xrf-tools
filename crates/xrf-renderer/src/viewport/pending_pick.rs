use std::sync::mpsc::Sender;

use xrf_error::XrfResult;

use crate::contract::render_level_hit::RenderLevelHit;

/// A pick waiting for its viewport's next frame: where, in CSS pixels from the viewport's corner, and who to tell.
pub struct PendingPick {
  pub x: f32,
  pub y: f32,
  pub reply: Sender<XrfResult<Option<RenderLevelHit>>>,
}
