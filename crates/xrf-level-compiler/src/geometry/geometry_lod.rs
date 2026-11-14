use crate::geometry::geometry_lod_vertex::GeometryLodVertex;

/// A multi-use reference's LOD, an `MT_LOD` node (`OGF_LOD`, `xrMU_Model_export_OGF.cpp:51`): the impostor's eight
/// quads placed by the reference, over the tree visuals the reference placed.
#[derive(Clone, Debug, PartialEq)]
pub struct GeometryLod {
  /// The reference, by index into `build.prj`'s multi-use references.
  pub reference: u32,
  /// The `build.prj` material its quads are drawn with.
  pub material: u32,
  pub sector: u16,
  /// The tree visuals it stands for, by index into [`crate::GeometryModels::trees`].
  pub trees: Vec<u32>,
  pub faces: [[GeometryLodVertex; 4]; 8],
}
