use serde::{Deserialize, Serialize};

use crate::contract::render_page_plane::RenderPagePlane;
use crate::contract::render_page_wash::RenderPageWash;

/// What the page shows where it is transparent around its viewports: the boxes painting there, and the wash the washed
/// ones carry.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderPageBackdrop {
  /// Outermost first, each painted over the ones before it; the first fills the window whatever its box.
  pub planes: Vec<RenderPagePlane>,
  pub wash: Option<RenderPageWash>,
}
