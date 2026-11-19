use xrf_math::{Vector2d, Vector3d};

use crate::mesh::mesh_vertex_bone::MeshVertexBone;

/// One vertex of a split: `SSkelVert` as `CExportSkeleton::PrepareGeometry` builds it, or the static export's
/// `SOGFVert`, following no bone. A level visual's lightmap coordinate and vertex colour are not held here.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshVertex {
  pub position: Vector3d,
  pub normal: Vector3d,
  /// The direction the texture's `u` runs in, about square to the normal: what bump mapping lights along.
  pub tangent: Vector3d,
  /// The direction the texture's `v` runs in, about square to the normal and the tangent ([`crate::TangentBasis`] says
  /// how far off).
  pub binormal: Vector3d,
  pub texture_coordinate: Vector2d,
  /// The bones it follows by ascending index, their weights normalised; none for a static vertex. On a face whose
  /// corners all follow one bone, the export pads the list with a weightless copy of that bone, so the vertex packs
  /// apart from the same corner of a blending face.
  pub bones: Vec<MeshVertexBone>,
}
