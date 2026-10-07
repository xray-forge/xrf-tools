use serde::{Deserialize, Serialize};

use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_surface_color::RenderSurfaceColor;

/// How a view shades what it shows: lit or as its albedo, filled or as its edges, what colours the surfaces, whether
/// bumps bend them, and which picture it shows.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderViewMode {
  /// Whether the scene is lit, else shown as its raw albedo.
  pub is_lit: bool,
  /// Whether every static surface draws as its triangles' edges.
  pub is_wireframe: bool,
  /// What colour surfaces' albedo is drawn with: their textures, clay, or their shader's tint.
  pub surface_color: RenderSurfaceColor,
  /// Whether bump textures bend the normal.
  pub is_bumped: bool,
  /// Which picture the viewport shows: its frame, or one of the targets the frame was built from.
  pub debug_view: RenderDebugView,
}

impl Default for RenderViewMode {
  fn default() -> Self {
    Self {
      is_lit: true,
      is_wireframe: false,
      surface_color: RenderSurfaceColor::Textured,
      is_bumped: true,
      debug_view: RenderDebugView::Final,
    }
  }
}
