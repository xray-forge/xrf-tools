use serde::{Deserialize, Serialize};

/// What a view's spot meter read at its centre, a square of nine texels a side averaged: the light there, what it was
/// exposed by, and what the G-buffer holds of the surfaces under it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderMeterReading {
  /// The scene's luminance at the spot before its exposure, in the units its light is drawn in.
  pub luminance: f32,
  /// Stops the exposed spot stands above middle grey, 0.18; below it where negative.
  pub stops: f32,
  /// The scale the frame's light was exposed by.
  pub exposure: f32,
  /// The surfaces' albedo luminance read as linear light (sRGB decoded); none where the spot holds only sky.
  pub albedo: Option<f32>,
  /// The surfaces' gloss, as the G-buffer keeps it.
  pub gloss: Option<f32>,
  /// The luminance of the diffuse light the sun and the lamps accumulated over the surfaces.
  pub diffuse: Option<f32>,
  /// The specular the sun and the lamps accumulated over the surfaces.
  pub specular: Option<f32>,
  /// Metres from the camera to the surfaces, along its view.
  pub distance: Option<f32>,
}
