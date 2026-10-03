use serde::{Deserialize, Serialize};

/// A checkerboard where nothing was drawn, as the one behind a picture with alpha: the backdrop and a second colour in
/// squares.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderBackdropSquares {
  /// The second colour, each channel zero to one.
  pub color: [f32; 3],
  /// Side of one square, in device pixels of the viewport.
  pub size: f32,
}
