use xrf_editable_object::{EditableMesh, EditableVMapKind};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Vector2d, Vector3d};

use crate::object::compile::geometry::export_vertex::ExportVertex;
use crate::object::compile::geometry::source::mesh_corner::MeshCorner;
use crate::object::compile::geometry::source::mesh_corners::MeshCorners;

/// One face corner as `CExportObjectOGF::PrepareMESH` sets its vertex: its point, its normal (see [`MeshCorners`])
/// and the first texture coordinate its vertex map reference names.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RigidCorner {
  pub(crate) position: Vector3d,
  pub(crate) normal: Vector3d,
  pub(crate) texture_coordinate: Vector2d,
}

impl RigidCorner {
  /// Each face's corners of `mesh`.
  ///
  /// # Errors
  ///
  /// Returns an error where [`MeshCorners`] refuses the mesh, and for a reference naming no UV map, which the export
  /// asserts against, or a UV entry of too few values.
  pub(crate) fn of_mesh(mesh: &EditableMesh) -> XrfResult<Vec<[Self; 3]>> {
    let corners: MeshCorners = MeshCorners::of(mesh)?;

    corners.read_faces(|corner| Self::of_corner(corner, corners.get_mesh_name()))
  }

  /// The vertex the corner is before packing: following no bone, its tangent and binormal what `SOGFVert` starts with
  /// until the tangent pass.
  pub(crate) fn to_vertex(&self) -> ExportVertex {
    ExportVertex {
      position: self.position,
      normal: self.normal,
      tangent: Vector3d::ZERO,
      binormal: Vector3d::ZERO,
      texture_coordinate: self.texture_coordinate,
      bones: Vec::new(),
    }
  }

  /// The corner with the first UV entry its reference names, past any other kind of entry.
  fn of_corner(corner: MeshCorner, mesh_name: &str) -> XrfResult<Self> {
    let entry = corner
      .entries
      .iter()
      .find(|entry| entry.vmap.kind == EditableVMapKind::Uv)
      .ok_or_else(|| XrfError::new_invalid_error(format!("A face corner of mesh '{mesh_name}' refers to no UV map")))?;
    let texture_coordinate: Vector2d = match entry.values {
      [u, v, ..] => Vector2d::new(*u, *v),
      _ => {
        return Err(XrfError::new_invalid_error(format!(
          "Vertex map '{}' holds {} values an entry, too few for a UV map",
          entry.vmap.name, entry.vmap.dimension
        )));
      }
    };

    Ok(Self {
      position: corner.position,
      normal: corner.normal,
      texture_coordinate,
    })
  }
}
