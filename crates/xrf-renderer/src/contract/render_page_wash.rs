use serde::{Deserialize, Serialize};

use crate::contract::render_rect::RenderRect;

/// A linear gradient over a box of the page, as CSS `linear-gradient(angle, from, to)` paints it over a colour.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPageWash {
  /// The box it is painted over, in device pixels of the window's client area.
  pub rect: RenderRect,
  /// Degrees clockwise from pointing up, as CSS states a gradient's angle.
  pub angle: f32,
  /// Its first and last colours, sRGB channels and alpha from 0 to 1.
  pub from: [f32; 4],
  pub to: [f32; 4],
}
