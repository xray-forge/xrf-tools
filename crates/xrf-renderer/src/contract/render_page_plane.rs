use serde::{Deserialize, Serialize};

use crate::contract::render_color::RenderColor;
use crate::contract::render_rect::RenderRect;

/// A box of the page painting a background under a viewport, which the renderer paints in its place.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPagePlane {
  /// The box its background shows in, cut to the boxes clipping it, in device pixels of the window's client area.
  pub rect: RenderRect,
  /// The radii its corners are rounded by in device pixels, in CSS's order: top left, top right, bottom right, bottom
  /// left.
  pub radii: [f32; 4],
  pub color: RenderColor,
  /// Whether the page's wash is laid over its colour.
  pub is_washed: bool,
}
