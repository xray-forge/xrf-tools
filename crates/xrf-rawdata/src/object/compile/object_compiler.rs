use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};

use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_geometry_builder::SkeletonGeometryBuilder;

/// Turns an editable object into what the Actor Editor's `Export OGF` writes, for the engine to load: see the crate's
/// documentation for what matches the editor's output and what only matches what the engine draws.
pub struct ObjectCompiler;

impl ObjectCompiler {
  /// The children a skeleton object's OGF holds, before tangents and the triangle order the engine draws in.
  ///
  /// # Errors
  ///
  /// Returns an error for an object that is not dynamic, which is not compiled yet, and where the Actor Editor's
  /// export fails (see [`SkeletonGeometry`]'s builder).
  pub fn build_skeleton_geometry(object: &EditableObjectFile) -> XrfResult<SkeletonGeometry> {
    if object.flags & EditableObjectFile::DYNAMIC_FLAG == 0 {
      return Err(XrfError::new_not_implemented_error(
        "A static object is not compiled yet; only skeletons, dynamic objects with bones, are",
      ));
    }

    SkeletonGeometryBuilder::build(object)
  }
}
