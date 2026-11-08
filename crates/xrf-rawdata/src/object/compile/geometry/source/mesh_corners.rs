use std::borrow::Cow;

use xrf_editable_object::{EditableFaceCorner, EditableMesh, EditableMeshNormals, EditableVMap};
use xrf_error::{XrfError, XrfResult};
use xrf_math::Vector3d;

use crate::object::compile::geometry::source::mesh_corner::MeshCorner;
use crate::object::compile::geometry::source::vertex_map_entry::VertexMapEntry;

/// A mesh's face corners as both exports read them (`CEditableMesh::GenerateSVertices` and `PrepareMESH`), every index
/// a face or a reference stores checked, so a broken `.object` is refused rather than read past its end.
///
/// A corner's normal is the one the mesh stores for it where it stores them, as `blender-xray` and the decompile do,
/// else the SDK's from the smoothing (`EditableMeshNormals`). The editors read no stored normals, so an object carrying
/// them compiles here to the normals it was made with, and in the SDK to its smoothing's.
pub(crate) struct MeshCorners<'a> {
  mesh: &'a EditableMesh,
  vmaps: &'a [EditableVMap],
  normals: Cow<'a, [[Vector3d; 3]]>,
}

impl<'a> MeshCorners<'a> {
  /// The corners of `mesh`.
  ///
  /// # Errors
  ///
  /// Returns an error when the mesh stores normals for another number of faces than it holds, or its smoothing does
  /// not hold together.
  pub(crate) fn of(mesh: &'a EditableMesh) -> XrfResult<Self> {
    let normals: Cow<'a, [[Vector3d; 3]]> = match &mesh.corner_normals {
      Some(stored) if stored.len() == mesh.faces.len() => Cow::Borrowed(stored),
      Some(stored) => {
        return Err(XrfError::new_invalid_error(format!(
          "Mesh '{}' stores {} corner normal triples for {} faces",
          mesh.name,
          stored.len(),
          mesh.faces.len()
        )));
      }
      None => Cow::Owned(EditableMeshNormals::of(mesh)?.corners),
    };

    Ok(Self {
      mesh,
      vmaps: mesh.vmaps.as_deref().unwrap_or_default(),
      normals,
    })
  }

  /// The corners of `mesh` for an export that reads no normal, as `EDetail::Update` reads none: each corner's normal is
  /// zero, and the smoothing is never walked.
  pub(crate) fn without_normals(mesh: &'a EditableMesh) -> Self {
    Self {
      mesh,
      vmaps: mesh.vmaps.as_deref().unwrap_or_default(),
      normals: Cow::Owned(vec![[Vector3d::ZERO; 3]; mesh.faces.len()]),
    }
  }

  /// Each face's three corners, each as `read` takes it.
  ///
  /// # Errors
  ///
  /// Returns an error when a face names a point or a reference the mesh does not have, a reference names a vertex map
  /// or an entry it does not have, or `read` fails.
  pub(crate) fn read_faces<T>(&self, read: impl Fn(MeshCorner<'a>) -> XrfResult<T>) -> XrfResult<Vec<[T; 3]>> {
    self
      .mesh
      .faces
      .iter()
      .zip(self.normals.iter())
      .map(|(face, normals)| {
        Ok([
          read(self.get_corner(&face.corners[0], normals[0])?)?,
          read(self.get_corner(&face.corners[1], normals[1])?)?,
          read(self.get_corner(&face.corners[2], normals[2])?)?,
        ])
      })
      .collect()
  }

  /// The mesh's name, which the exports' errors name.
  pub(crate) fn get_mesh_name(&self) -> &str {
    &self.mesh.name
  }

  fn get_corner(&self, corner: &EditableFaceCorner, normal: Vector3d) -> XrfResult<MeshCorner<'a>> {
    let mesh: &'a EditableMesh = self.mesh;
    let position: Vector3d = *mesh
      .points
      .get(corner.point as usize)
      .ok_or_else(|| XrfError::new_invalid_error(format!("Mesh '{}' names point {}", mesh.name, corner.point)))?;
    let reference = mesh
      .vmap_references
      .get(corner.vmap_reference as usize)
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Mesh '{}' names vertex map reference {}",
          mesh.name, corner.vmap_reference
        ))
      })?;
    let entries: Vec<VertexMapEntry<'a>> = reference
      .points
      .iter()
      .map(|entry| {
        let vmap: &'a EditableVMap = self.vmaps.get(entry.vmap as usize).ok_or_else(|| {
          XrfError::new_invalid_error(format!("Mesh '{}' names vertex map {}", mesh.name, entry.vmap))
        })?;
        let values: &'a [f32] = vmap.get_entry(entry.index as usize).ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "Mesh '{}' names entry {} of vertex map '{}'",
            mesh.name, entry.index, vmap.name
          ))
        })?;

        Ok(VertexMapEntry { vmap, values })
      })
      .collect::<XrfResult<_>>()?;

    Ok(MeshCorner {
      position,
      normal,
      entries,
    })
  }
}
