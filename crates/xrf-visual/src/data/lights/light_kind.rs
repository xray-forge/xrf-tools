use serde::Serialize;

/// The shape a light reaches out in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LightKind {
  /// Every way around it, to its range.
  Point,
  /// Along its direction, within its cone, through its projector.
  Spot,
}
