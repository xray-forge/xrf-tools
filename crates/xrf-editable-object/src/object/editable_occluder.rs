/// What an object flagged as an occluder blocks, which the Build writes it out as instead of level geometry, and which
/// the Level Editor draws it as instead of its surfaces (`CEditableObject::Render`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditableOccluder {
  /// `eoHOM`: sight, written to `level.hom`.
  Hom,
  /// `eoSoundOccluder`: sound, written to `level.som`.
  Sound,
}
