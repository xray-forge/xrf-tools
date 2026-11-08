use crate::object::compile::geometry::export_vertex::ExportVertex;

/// One child of a skeleton OGF before it is written: the faces of every surface sharing an engine shader, a texture and
/// a breakable part (`CExportSkeleton::SSplit`), their corners packed into vertices.
#[derive(Clone, Debug, PartialEq)]
pub struct SkeletonSplit {
  pub shader: String,
  pub texture: String,
  /// The breakable part its bones belong to, 0 for an object with no breakable bone.
  pub part: u16,
  /// Links its widest vertex stores, one to four: the layout every vertex of the child is written in.
  pub link_count: u8,
  /// The bones its vertices follow, ascending.
  pub used_bones: Vec<u16>,
  pub vertices: Vec<ExportVertex>,
  /// Each face's vertices, in the winding it was authored in.
  pub faces: Vec<[u16; 3]>,
}
