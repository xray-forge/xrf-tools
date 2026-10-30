use serde::{Deserialize, Serialize};

/// Which of a level's helpers a view draws and how: the kinds switched on, the objects hidden and selected, and how an
/// occluded wire shows.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderHelperOptions {
  /// Whether helpers draw at all.
  pub is_shown: bool,
  /// The kinds drawn, a bit a kind as the helpers number them.
  pub kinds: u32,
  /// The objects drawn nothing of, by the ids the helpers carry, ascending.
  pub hidden: Vec<u32>,
  /// The objects drawn as selected, never faded, by the ids the helpers carry, ascending.
  pub selected: Vec<u32>,
  /// How strongly a wire behind what is drawn shows, from 0 to 1.
  pub occluded_strength: f32,
  /// What every kind's draw distance is scaled by.
  pub distance_scale: f32,
}

impl Default for RenderHelperOptions {
  fn default() -> Self {
    Self {
      is_shown: false,
      kinds: 0,
      hidden: Vec::new(),
      selected: Vec::new(),
      occluded_strength: 0.25,
      distance_scale: 1.0,
    }
  }
}
