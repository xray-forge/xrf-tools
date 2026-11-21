use xrf_editable_level::EditableSceneObject;
use xrf_editable_object::{EditableMesh, EditableObjectFile, EditableSurface, EditableSurfaceFaces};
use xrf_error::{XrfError, XrfResult};

/// A placed object's surfaces, found by the names its meshes' face groups give them, as every walk of a build's
/// objects reads them.
pub(crate) struct LevelBuildSurfaces<'a> {
  reference: &'a str,
  surfaces: &'a [EditableSurface],
}

impl<'a> LevelBuildSurfaces<'a> {
  pub(crate) fn of(placed: &'a EditableSceneObject, object: &'a EditableObjectFile) -> Self {
    Self {
      reference: &placed.reference,
      surfaces: object.surfaces.as_deref().unwrap_or_default(),
    }
  }

  /// Each face group of `mesh` with the surface it names, in the mesh's order of groups.
  ///
  /// # Errors
  ///
  /// Yields an error, as the Build stops, for a group naming a surface the object does not hold.
  pub(crate) fn iter_groups<'m>(
    &self,
    mesh: &'m EditableMesh,
  ) -> impl Iterator<Item = XrfResult<(&'a EditableSurface, &'m EditableSurfaceFaces)>> {
    let (reference, surfaces): (&'a str, &'a [EditableSurface]) = (self.reference, self.surfaces);

    mesh.surface_faces.iter().map(move |group| {
      surfaces
        .iter()
        .find(|surface| surface.name == group.surface)
        .map(|surface| (surface, group))
        .ok_or_else(|| {
          XrfError::new_invalid_error(format!(
            "Object '{reference}' mesh '{}' names surface '{}' it does not hold",
            mesh.name, group.surface
          ))
        })
    })
  }
}
