use serde::{Deserialize, Serialize};

use crate::contract::render_page_backdrop::RenderPageBackdrop;
use crate::contract::render_rect::RenderRect;

/// Where a viewport sits in its window and what the page shows around it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewportLayout {
  /// The viewport element's rectangle, in device pixels of the window's client area.
  pub rect: RenderRect,
  /// Device pixels per CSS pixel, the unit input coordinates are given in.
  pub scale: f32,
  /// What the page shows where it is transparent, painted under the viewports: around them, and where one has not
  /// followed a layout change yet.
  pub backdrop: RenderPageBackdrop,
}
