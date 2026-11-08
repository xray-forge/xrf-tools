use std::borrow::Cow;
use std::collections::HashMap;

use xrf_editable_object::{EditableMesh, EditableMeshNormals, EditableVMap, EditableVMapKind};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Vector2d, Vector3d};

use crate::object::compile::skeleton::skeleton_vertex::SkeletonVertexBone;

/// One face corner as `CEditableMesh::GenerateSVertices` gives it the export: its point, its normal (see
/// [`Self::get_corner_normals`]), the texture coordinate and the bone weights its vertex map reference names.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SkinCorner {
  pub(crate) position: Vector3d,
  pub(crate) normal: Vector3d,
  pub(crate) texture_coordinate: Vector2d,
  /// Strongest first, at most [`Self::INFLUENCE`], summing to one (`st_VertexWB::prepare_weights`).
  pub(crate) weights: Vec<SkeletonVertexBone>,
}

impl SkinCorner {
  /// Bones a corner keeps, the strongest: the Actor Editor exports with an influence of four.
  pub(crate) const INFLUENCE: usize = 4;

  /// Each face's corners of `mesh`, weights resolved to bones through `bone_of_map`, a weight map's name to the bone
  /// whose own weight map it is (`GetBoneIndexByWMap`).
  ///
  /// # Errors
  ///
  /// Returns an error when a face names a point or a reference the mesh does not have, a reference names a vertex map
  /// or an entry it does not have, or a weight map belongs to no bone.
  pub(crate) fn of_mesh(mesh: &EditableMesh, bone_of_map: &HashMap<&str, u16>) -> XrfResult<Vec<[Self; 3]>> {
    let normals: Cow<[[Vector3d; 3]]> = Self::get_corner_normals(mesh)?;
    let vmaps: &[EditableVMap] = mesh.vmaps.as_deref().unwrap_or_default();

    mesh
      .faces
      .iter()
      .zip(normals.iter())
      .map(|(face, corner_normals)| {
        let corner = |index: usize| -> XrfResult<Self> {
          let point = face.corners[index];

          Self::of_reference(mesh, vmaps, bone_of_map, point.vmap_reference).map(|(texture_coordinate, weights)| Self {
            position: mesh.points[point.point as usize],
            normal: corner_normals[index],
            texture_coordinate,
            weights,
          })
        };

        Ok([corner(0)?, corner(1)?, corner(2)?])
      })
      .collect()
  }

  /// Each face's corner normals: those the mesh stores where it stores them, as `blender-xray` and the decompile do,
  /// else the SDK's from the smoothing (`EditableMeshNormals`). The Actor Editor reads no stored normals, so an object
  /// carrying them compiles here to the normals it was made with, and in the SDK to its smoothing's.
  fn get_corner_normals(mesh: &EditableMesh) -> XrfResult<Cow<'_, [[Vector3d; 3]]>> {
    match &mesh.corner_normals {
      Some(stored) if stored.len() == mesh.faces.len() => Ok(Cow::Borrowed(stored)),
      Some(stored) => Err(XrfError::new_invalid_error(format!(
        "Mesh '{}' stores {} corner normal triples for {} faces",
        mesh.name,
        stored.len(),
        mesh.faces.len()
      ))),
      None => Ok(Cow::Owned(EditableMeshNormals::of(mesh)?.corners)),
    }
  }

  /// The texture coordinate, the last a reference names as the export keeps it, and the prepared weights.
  fn of_reference(
    mesh: &EditableMesh,
    vmaps: &[EditableVMap],
    bone_of_map: &HashMap<&str, u16>,
    reference: u32,
  ) -> XrfResult<(Vector2d, Vec<SkeletonVertexBone>)> {
    let entries = &mesh
      .vmap_references
      .get(reference as usize)
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!("Mesh '{}' names vertex map reference {reference}", mesh.name))
      })?
      .points;
    let mut texture_coordinate: Vector2d = Vector2d::ZERO;
    let mut weights: Vec<SkeletonVertexBone> = Vec::with_capacity(entries.len());

    for entry in entries {
      let vmap: &EditableVMap = vmaps
        .get(entry.vmap as usize)
        .ok_or_else(|| XrfError::new_invalid_error(format!("Mesh '{}' names vertex map {}", mesh.name, entry.vmap)))?;
      let values: &[f32] = vmap.get_entry(entry.index as usize).ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Mesh '{}' names entry {} of vertex map '{}'",
          mesh.name, entry.index, vmap.name
        ))
      })?;

      match (vmap.kind, values) {
        (EditableVMapKind::Uv, [u, v, ..]) => texture_coordinate = Vector2d::new(*u, *v),
        (EditableVMapKind::Weight, [weight, ..]) => weights.push(SkeletonVertexBone {
          bone: *bone_of_map
            .get(vmap.name.as_str())
            .ok_or_else(|| XrfError::new_invalid_error(format!("Weight map '{}' belongs to no bone", vmap.name)))?,
          weight: *weight,
        }),
        _ => {
          return Err(XrfError::new_invalid_error(format!(
            "Vertex map '{}' holds {} values an entry, too few for its kind",
            vmap.name, vmap.dimension
          )));
        }
      }
    }

    // Strongest first, keeping the order of equals as they were named.
    weights.sort_by(|first, second| second.weight.total_cmp(&first.weight));
    weights.truncate(Self::INFLUENCE);

    let sum: f32 = weights.iter().map(|bone| bone.weight).sum();

    for bone in &mut weights {
      bone.weight /= sum;
    }

    Ok((texture_coordinate, weights))
  }
}
