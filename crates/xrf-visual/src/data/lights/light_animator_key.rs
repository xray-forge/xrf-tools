use serde::Serialize;

/// One key of a colour animation: the frame it stands at and its colour, each channel in `[0, 255]`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightAnimatorKey {
  pub frame: u32,
  pub color: [f32; 3],
}
