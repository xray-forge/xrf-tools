use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};

use crate::object::compile::object_compile_options::ObjectCompileOptions;
use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_geometry_builder::SkeletonGeometryBuilder;
use crate::object::compile::skeleton::skeleton_ogf_writer::SkeletonOgfWriter;

/// Turns an editable object into what the Actor Editor's `Export OGF` writes, for the engine to load: see the crate's
/// documentation for what matches the editor's output and what only matches what the engine draws.
pub struct ObjectCompiler;

impl ObjectCompiler {
  /// The OGF of a skeleton object, as bytes.
  ///
  /// # Errors
  ///
  /// Returns an error where [`Self::build_skeleton_geometry`] does, and for an object embedding its own motions, which
  /// is not compiled yet.
  pub fn compile(object: &EditableObjectFile, source: &ObjectCompileSource) -> XrfResult<Vec<u8>> {
    Self::compile_with(object, source, &ObjectCompileOptions::default())
  }

  /// [`Self::compile`] with `options` for the choices the editor's export leaves open.
  ///
  /// # Errors
  ///
  /// As [`Self::compile`].
  pub fn compile_with(
    object: &EditableObjectFile,
    source: &ObjectCompileSource,
    options: &ObjectCompileOptions,
  ) -> XrfResult<Vec<u8>> {
    Self::write_skeleton(object, &Self::build_skeleton_geometry_with(object, options)?, source)
  }

  /// The OGF of a skeleton object whose `geometry` [`Self::build_skeleton_geometry`] built, as bytes.
  ///
  /// # Errors
  ///
  /// Returns an error for an object embedding its own motions, which is not compiled yet, and where a chunk does not
  /// write, such as a description time past the 32-bit times it was authored with.
  pub fn write_skeleton(
    object: &EditableObjectFile,
    geometry: &SkeletonGeometry,
    source: &ObjectCompileSource,
  ) -> XrfResult<Vec<u8>> {
    SkeletonOgfWriter::write::<XRayByteOrder>(object, geometry, source)
  }

  /// The children a skeleton object's OGF holds, before the triangle order the engine draws in.
  ///
  /// # Errors
  ///
  /// Returns an error for an object that is not dynamic, which is not compiled yet, and where the Actor Editor's
  /// export fails (see [`SkeletonGeometry`]'s builder).
  pub fn build_skeleton_geometry(object: &EditableObjectFile) -> XrfResult<SkeletonGeometry> {
    Self::build_skeleton_geometry_with(object, &ObjectCompileOptions::default())
  }

  /// [`Self::build_skeleton_geometry`] with `options` for the choices the editor's export leaves open.
  ///
  /// # Errors
  ///
  /// As [`Self::build_skeleton_geometry`].
  pub fn build_skeleton_geometry_with(
    object: &EditableObjectFile,
    options: &ObjectCompileOptions,
  ) -> XrfResult<SkeletonGeometry> {
    if object.flags & EditableObjectFile::DYNAMIC_FLAG == 0 {
      return Err(XrfError::new_not_implemented_error(
        "A static object is not compiled yet; only skeletons, dynamic objects with bones, are",
      ));
    }

    SkeletonGeometryBuilder::build(object, options)
  }
}
