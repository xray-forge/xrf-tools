use serde::{Deserialize, Serialize};

use crate::contract::render_applied_report::RenderAppliedReport;
use crate::contract::render_frame_report::RenderFrameReport;
use crate::contract::render_load_report::RenderLoadReport;

/// What the renderer tells a viewport's page.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RenderViewportEvent {
  /// What the recent frames cost.
  Frame { report: RenderFrameReport },
  /// What its frames are drawn with, as the renderer resolved what it was asked, sent as it changes.
  Applied { report: RenderAppliedReport },
  /// How far its scene has loaded, sent as it changes.
  Load { report: RenderLoadReport },
  /// The renderer cannot draw this viewport, and why.
  Failure { message: String },
}
