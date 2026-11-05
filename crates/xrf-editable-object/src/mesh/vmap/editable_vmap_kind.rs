/// What a vertex map holds, `vmtUV` or `vmtWeight`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditableVMapKind {
  /// Two texture coordinates per entry.
  Uv,
  /// One bone weight per entry; the map's name is the bone's weight map.
  Weight,
  /// A kind the SDK does not name, kept so it writes back as stored.
  Unknown(u8),
}

impl From<u8> for EditableVMapKind {
  fn from(value: u8) -> Self {
    match value {
      0 => Self::Uv,
      1 => Self::Weight,
      other => Self::Unknown(other),
    }
  }
}

impl From<EditableVMapKind> for u8 {
  fn from(kind: EditableVMapKind) -> Self {
    match kind {
      EditableVMapKind::Uv => 0,
      EditableVMapKind::Weight => 1,
      EditableVMapKind::Unknown(value) => value,
    }
  }
}
