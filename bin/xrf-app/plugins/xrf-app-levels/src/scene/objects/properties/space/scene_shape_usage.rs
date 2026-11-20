use serde::Serialize;
use xrf_editable_level::EditableShape;

/// What a shape is used for, `eShapeUsage`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum SceneShapeUsage {
  /// `eShapeCommon`.
  Common,
  /// `eShapeLevelBound`: the box the build takes for the level's map.
  LevelBound,
  /// A usage the editor gives no name, kept as the number the part saves.
  Unknown { value: u8 },
}

impl SceneShapeUsage {
  /// `eShapeCommon`.
  const COMMON: u8 = 0;

  /// The usage a shape's part saves as `shape_type`.
  pub(crate) const fn of(usage: u8) -> Self {
    match usage {
      Self::COMMON => Self::Common,
      EditableShape::LEVEL_BOUND_USAGE => Self::LevelBound,
      value => Self::Unknown { value },
    }
  }
}
