use serde::{Deserialize, Serialize};

use crate::contract::render_camera_pose::RenderCameraPose;
use crate::contract::render_frame_report::RenderFrameReport;

/// What a viewport tells its page.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RenderViewportEvent {
  /// What the recent frames cost.
  Frame { report: RenderFrameReport },
  /// Where the camera stands, sent while it moves and once more after it stops.
  Camera { pose: RenderCameraPose },
  /// The renderer cannot draw this viewport, and why.
  Failure { message: String },
}
