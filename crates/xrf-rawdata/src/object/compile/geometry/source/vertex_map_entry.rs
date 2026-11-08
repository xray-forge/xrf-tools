use xrf_editable_object::EditableVMap;

/// One entry a face corner's vertex map reference names: the map, and the values it holds for the corner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct VertexMapEntry<'a> {
  pub(crate) vmap: &'a EditableVMap,
  /// As many values as the map's dimension.
  pub(crate) values: &'a [f32],
}
