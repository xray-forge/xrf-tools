/// What a visual is lit by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GeometryVisualLight {
  /// Its vertices' colour, the compiler shader's `bLIGHT_Vertex`.
  Vertex,
  /// A lightmap, or an implicitly lit texture's map.
  Lightmap,
  /// Its placements' tree colours, a multi-use model's visual.
  Tree,
}
