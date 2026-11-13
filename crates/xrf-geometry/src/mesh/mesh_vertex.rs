use xrf_math::{Vector2d, Vector3d};

use crate::mesh::mesh_vertex_bone::MeshVertexBone;

/// One vertex of a split, as the passes of this module pack, mend and cut it: `SSkelVert` as
/// `CExportSkeleton::PrepareGeometry` builds it, or `SOGFVert` as the static export does, following no bone. A level's
/// visuals carry a lightmap coordinate or a vertex colour beside these, which join it with the level compiler's first
/// pass that reads them.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshVertex {
  pub position: Vector3d,
  pub normal: Vector3d,
  /// The direction the texture's `u` runs in, at right angles to the normal: what bump mapping lights along.
  pub tangent: Vector3d,
  /// The direction the texture's `v` runs in, at right angles to the normal and the tangent.
  pub binormal: Vector3d,
  pub texture_coordinate: Vector2d,
  /// The bones it follows by ascending index, their weights normalised; none for a static vertex. A skeleton vertex of
  /// a face whose corners all follow one bone lists that bone twice, the copy weightless, as the export pads it; it then
  /// packs apart from the same corner of a face that blends, which lists the bone once.
  pub bones: Vec<MeshVertexBone>,
}
