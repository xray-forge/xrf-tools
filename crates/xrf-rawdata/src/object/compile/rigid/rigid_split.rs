use crate::object::compile::geometry::export_vertex::ExportVertex;

/// One child of a static object's OGF: a surface's vertices and faces, as `CExportObjectOGF::SSplit::SavePart` writes
/// a part.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RigidSplit {
  pub(crate) texture: String,
  pub(crate) shader: String,
  /// Its vertices as the tangent pass split and ordered them, each following no bone; the OGF keeps their positions,
  /// normals and texture coordinates.
  pub(crate) vertices: Vec<ExportVertex>,
  pub(crate) faces: Vec<[u16; 3]>,
}
