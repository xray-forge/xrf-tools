use serde::{Deserialize, Serialize};

use crate::contract::render_color::RenderColor;
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
  /// The page's background around the viewport, cleared where the page is transparent but the viewport has not
  /// followed a layout change yet.
  pub clear: RenderColor,
}
