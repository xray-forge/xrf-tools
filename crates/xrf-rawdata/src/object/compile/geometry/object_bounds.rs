use xrf_editable_object::EditableMesh;
use xrf_math::Aabb;

/// The box an object's vertex packing spans, `CEditableObject::UpdateBox`.
pub(crate) struct ObjectBounds;

impl ObjectBounds {
  /// Every mesh's stored bounds together, or `None` for an object without meshes.
  pub(crate) fn of(meshes: &[EditableMesh]) -> Option<Aabb> {
    meshes.iter().map(|mesh| mesh.bounds).reduce(|bounds, other| Aabb {
      min: bounds.min.min(&other.min),
      max: bounds.max.max(&other.max),
    })
  }
}
