use serde::Serialize;
use xrf_editable_level::EditableLightObject;

/// What kind of light a light is, `ELight::EType`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum SceneLightType {
  /// `ltPoint`, the only kind the build exports.
  Point,
  /// `ltSpot`.
  Spot,
  /// `ltDirect`.
  Directional,
  /// A type the editor gives no name, kept as the number the part saves.
  Unknown { value: u32 },
}

impl SceneLightType {
  /// `ltSpot`, `D3DLIGHT_SPOT`.
  const SPOT: u32 = 2;
  /// `ltDirect`, `D3DLIGHT_DIRECTIONAL`.
  const DIRECTIONAL: u32 = 3;

  /// The type a light's part saves as `kind`.
  pub(crate) const fn of(kind: u32) -> Self {
    match kind {
      EditableLightObject::POINT => Self::Point,
      Self::SPOT => Self::Spot,
      Self::DIRECTIONAL => Self::Directional,
      value => Self::Unknown { value },
    }
  }
}
