/// One corner of a face, `st_FaceVert`: the point it sits on and the list of vertex map entries it takes its UV and
/// weights from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditableFaceCorner {
  /// Index into the mesh's points.
  pub point: u32,
  /// Index into the mesh's vertex map references.
  pub vmap_reference: u32,
}
