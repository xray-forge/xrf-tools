use serde::{Deserialize, Serialize};

use crate::contract::render_color::RenderColor;
use crate::contract::render_page_wash::RenderPageWash;

/// What the page shows where it is transparent around its viewports: its colour, and the wash laid over it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPageBackdrop {
  pub color: RenderColor,
  pub wash: Option<RenderPageWash>,
}
