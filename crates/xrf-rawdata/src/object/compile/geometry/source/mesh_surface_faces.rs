use xrf_editable_object::{EditableMesh, EditableSurface};
use xrf_error::{XrfError, XrfResult};

/// One surface's faces of a mesh, as an export walks them.
pub(crate) struct MeshSurfaceFaces<'a> {
  /// The surface's place among the object's surfaces.
  pub(crate) index: usize,
  pub(crate) surface: &'a EditableSurface,
  /// Its faces, by index into the mesh's.
  pub(crate) faces: &'a [u32],
}

impl<'a> MeshSurfaceFaces<'a> {
  /// `mesh`'s face lists in the order an export walks them: the editor keys a mesh's faces by surface pointer, so walks
  /// them in the order it allocated the surfaces, the object's `surfaces`.
  ///
  /// # Errors
  ///
  /// Returns an error where the export fails: a face list naming a surface the object does not have, a surface whose
  /// vertex format names other than one texture coordinate set, or a face past the mesh's.
  pub(crate) fn in_export_order(mesh: &'a EditableMesh, surfaces: &'a [EditableSurface]) -> XrfResult<Vec<Self>> {
    let mut walked: Vec<Self> = mesh
      .surface_faces
      .iter()
      .map(|surface_faces| {
        let index: usize = surfaces
          .iter()
          .position(|surface| surface.name == surface_faces.surface)
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "Mesh '{}' names surface '{}', which the object does not have",
              mesh.name, surface_faces.surface
            ))
          })?;
        let surface: &EditableSurface = &surfaces[index];

        if surface.get_fvf_texture_sets() != 1 {
          return Err(XrfError::new_invalid_error(format!(
            "Surface '{}' names other than one texture coordinate set, which the export requires",
            surface.name
          )));
        }

        if let Some(face) = surface_faces
          .faces
          .iter()
          .find(|face| **face as usize >= mesh.faces.len())
        {
          return Err(XrfError::new_invalid_error(format!(
            "Surface '{}' names face {face} of mesh '{}'",
            surface.name, mesh.name
          )));
        }

        Ok(Self {
          index,
          surface,
          faces: &surface_faces.faces,
        })
      })
      .collect::<XrfResult<_>>()?;

    walked.sort_by_key(|it| it.index);

    Ok(walked)
  }
}
