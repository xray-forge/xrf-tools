use std::sync::mpsc::Sender;

use xrf_error::XrfResult;

use crate::contract::render_pick::RenderPick;

/// A pick waiting for its viewport's next frame: where, in CSS pixels from the viewport's corner, whether it passes the
/// composited surfaces by, and who to tell.
pub struct PendingPick {
  pub x: f32,
  pub y: f32,
  pub is_through: bool,
  pub reply: Sender<XrfResult<RenderPick>>,
}
