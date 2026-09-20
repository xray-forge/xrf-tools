use serde::Serialize;

use crate::data::visual::geometry::visual_skip_cause::VisualSkipCause;

/// A drawable of a sector that produced no geometry, and why.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SectorSkip {
  /// The visual left out, by its index in the visuals run.
  pub drawable: u32,
  pub cause: VisualSkipCause,
  pub reason: String,
}
