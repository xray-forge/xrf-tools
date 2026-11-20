use serde::Serialize;
use xrf_editable_level::EditableWayObject;

/// What a way is, `EWayType`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum SceneWayType {
  /// `wtPatrolPath`, every way the editor makes.
  PatrolPath,
  /// A type the editor gives no name, kept as the number the part saves.
  Unknown { value: u32 },
}

impl SceneWayType {
  /// The type a way's part saves as `kind`.
  pub(crate) const fn of(kind: u32) -> Self {
    match kind {
      EditableWayObject::PATROL_PATH => Self::PatrolPath,
      value => Self::Unknown { value },
    }
  }
}
