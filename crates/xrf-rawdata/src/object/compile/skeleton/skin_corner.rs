use std::collections::HashMap;

use xrf_editable_object::{EditableMesh, EditableVMapKind};
use xrf_error::{XrfError, XrfResult};
use xrf_math::{Vector2d, Vector3d};

use crate::object::compile::geometry::export_vertex_bone::ExportVertexBone;
use crate::object::compile::geometry::source::mesh_corner::MeshCorner;
use crate::object::compile::geometry::source::mesh_corners::MeshCorners;

/// One face corner as `CEditableMesh::GenerateSVertices` gives it the skeleton export: its point, its normal (see
/// [`MeshCorners`]), the last texture coordinate and the bone weights its vertex map reference names.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SkinCorner {
  pub(crate) position: Vector3d,
  pub(crate) normal: Vector3d,
  pub(crate) texture_coordinate: Vector2d,
  /// Strongest first, at most [`Self::INFLUENCE`], summing to one (`st_VertexWB::prepare_weights`).
  pub(crate) weights: Vec<ExportVertexBone>,
}

impl SkinCorner {
  /// Bones a corner keeps, the strongest: the Actor Editor exports with an influence of four.
  pub(crate) const INFLUENCE: usize = 4;

  /// Each face's corners of `mesh`, weights resolved to bones through `bone_of_map`, a weight map's name to the bone
  /// whose own weight map it is (`GetBoneIndexByWMap`).
  ///
  /// # Errors
  ///
  /// Returns an error where [`MeshCorners`] refuses the mesh, an entry holds too few values for its map's kind, or a
  /// weight map belongs to no bone.
  pub(crate) fn of_mesh(mesh: &EditableMesh, bone_of_map: &HashMap<&str, u16>) -> XrfResult<Vec<[Self; 3]>> {
    MeshCorners::of(mesh)?.read_faces(|corner| Self::of_corner(corner, bone_of_map))
  }

  /// The corner with the last UV its reference names, as the export keeps it, and its weights prepared.
  fn of_corner(corner: MeshCorner<'_>, bone_of_map: &HashMap<&str, u16>) -> XrfResult<Self> {
    let mut texture_coordinate: Vector2d = Vector2d::ZERO;
    let mut weights: Vec<ExportVertexBone> = Vec::with_capacity(corner.entries.len());

    for entry in &corner.entries {
      match (entry.vmap.kind, entry.values) {
        (EditableVMapKind::Uv, [u, v, ..]) => texture_coordinate = Vector2d::new(*u, *v),
        (EditableVMapKind::Weight, [weight, ..]) => weights.push(ExportVertexBone {
          bone: *bone_of_map.get(entry.vmap.name.as_str()).ok_or_else(|| {
            XrfError::new_invalid_error(format!("Weight map '{}' belongs to no bone", entry.vmap.name))
          })?,
          weight: *weight,
        }),
        _ => {
          return Err(XrfError::new_invalid_error(format!(
            "Vertex map '{}' holds {} values an entry, too few for its kind",
            entry.vmap.name, entry.vmap.dimension
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

    Ok(Self {
      position: corner.position,
      normal: corner.normal,
      texture_coordinate,
      weights,
    })
  }
}
