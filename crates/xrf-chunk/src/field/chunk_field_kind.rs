/// The primitive a [`crate::ChunkField`] was written with, `NET_Packet`'s `w_*` of that name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkFieldKind {
  U8,
  U16,
  U32,
  U64,
  F32,
  /// Three floats as one value, `w_vec3`.
  Vector3,
  /// A null-terminated Windows-1251 string, `w_stringZ`.
  String,
}
