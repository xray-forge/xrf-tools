/// What a node of a level's visual tree is (`g_tree`'s `OGF_Base` kinds).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GeometryNodeKind {
  /// A static visual, by index into [`crate::geometry::visual::geometry_visuals::GeometryVisuals::visuals`].
  Visual(u32),
  /// A multi-use placement, by index into [`crate::geometry::model::geometry_models::GeometryModels::trees`].
  Tree(u32),
  /// A LOD node over its trees, by index into [`crate::geometry::model::geometry_models::GeometryModels::lods`].
  Lod(u32),
  /// A hierarchy node over other nodes (`OGF_Node`, `MT_HIERRARHY`), by index into the tree.
  Hierarchy(Vec<u32>),
}
