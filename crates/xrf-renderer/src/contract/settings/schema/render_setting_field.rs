use serde::Serialize;

/// What one of the settings takes, which the page offers and holds a stored value to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RenderSettingField {
  /// On or off.
  Flag,
  /// A number between bounds.
  Number(RenderSettingNumber),
  /// One of a set of values, as each serializes.
  Choice { values: Vec<String> },
  /// A run of positive widths, each between bounds, as many as the most given, as the sun's cascades are.
  Widths {
    /// The least each width may be.
    min: f32,
    /// The most each width may be.
    max: f32,
    /// The most widths the run holds.
    most: u32,
  },
}

/// A number the settings take between bounds, held to them: the engine console's own where the engine has the
/// setting (`xrRender_console.cpp`), the renderer's where it does not.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSettingNumber {
  pub min: f32,
  pub max: f32,
  /// Whether it is whole, as an engine's `CCC_Integer` is.
  pub is_integer: bool,
  /// The range xrf recommends within the bounds, Base's and Enhanced's values within it: below it the effect shows
  /// misses or artefacts, past it it costs more for little visible gain; none where it is not rated.
  pub recommended: Option<[f32; 2]>,
  /// The value past which a higher one was measured to look the same and only cost more; none where it was not.
  pub wasted_above: Option<f32>,
}
