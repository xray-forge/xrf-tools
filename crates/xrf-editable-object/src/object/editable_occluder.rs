/// What an object flagged as an occluder blocks. The Build writes it as that instead of level geometry, and the Level
/// Editor draws it so instead of its surfaces (`CEditableObject::Render`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditableOccluder {
  /// `eoHOM`: sight, written to `level.hom`.
  Hom,
  /// `eoSoundOccluder`: sound, written to `level.som`.
  Sound,
}
