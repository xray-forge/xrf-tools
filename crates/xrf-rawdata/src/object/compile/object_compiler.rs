use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;

use crate::object::compile::object_compile::ObjectCompile;
use crate::object::compile::object_compile_options::ObjectCompileOptions;
use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::rigid::rigid_geometry::RigidGeometry;
use crate::object::compile::rigid::rigid_geometry_builder::RigidGeometryBuilder;
use crate::object::compile::rigid::rigid_ogf_writer::RigidOgfWriter;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_geometry_builder::SkeletonGeometryBuilder;
use crate::object::compile::skeleton::skeleton_ogf_writer::SkeletonOgfWriter;

/// Turns an editable object into what the Actor Editor's `Export OGF` writes, for the engine to load: see the crate's
/// documentation for what matches the editor's output and what only matches what the engine draws.
pub struct ObjectCompiler;

impl ObjectCompiler {
  /// The OGF of an object: a skeleton's where it has bones, a static visual's otherwise, as
  /// `CEditableObject::PrepareOGF` chooses.
  ///
  /// # Errors
  ///
  /// Returns an error where the export fails (see [`Self::build_skeleton_geometry`] for a skeleton), and where a chunk
  /// does not write.
  pub fn compile(object: &EditableObjectFile, source: &ObjectCompileSource) -> XrfResult<ObjectCompile> {
    Self::compile_with(object, source, &ObjectCompileOptions::default())
  }

  /// [`Self::compile`] with `options` for the choices the editor's export leaves open; a static visual stores no
  /// tangents, so takes no tangent method.
  ///
  /// # Errors
  ///
  /// As [`Self::compile`].
  pub fn compile_with(
    object: &EditableObjectFile,
    source: &ObjectCompileSource,
    options: &ObjectCompileOptions,
  ) -> XrfResult<ObjectCompile> {
    if Self::is_skeleton(object) {
      let geometry: SkeletonGeometry = Self::build_skeleton_geometry_with(object, options)?;

      Ok(ObjectCompile {
        ogf: Self::write_skeleton(object, &geometry, source)?,
        children: geometry.splits.len(),
        vertices: geometry.splits.iter().map(|split| split.vertices.len()).sum(),
        faces: geometry.splits.iter().map(|split| split.faces.len()).sum(),
        dropped_faces: geometry.dropped_faces,
        tangents: Some(options.tangents),
      })
    } else {
      let geometry: RigidGeometry = RigidGeometryBuilder::build(object)?;

      Ok(ObjectCompile {
        ogf: RigidOgfWriter::write::<XRayByteOrder>(object, &geometry, source)?,
        children: geometry.splits.len(),
        vertices: geometry.splits.iter().map(|split| split.vertices.len()).sum(),
        faces: geometry.splits.iter().map(|split| split.faces.len()).sum(),
        dropped_faces: geometry.dropped_faces,
        tangents: None,
      })
    }
  }

  /// The OGF of a skeleton object whose `geometry` [`Self::build_skeleton_geometry`] built, as bytes.
  ///
  /// # Errors
  ///
  /// Returns an error where the object's own motions do not export, and where a chunk does not write, such as a
  /// description time past the 32-bit times it was authored with.
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
  /// Returns an error where the Actor Editor's export fails (see [`SkeletonGeometry`]'s builder), an object without
  /// bones among them.
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
    SkeletonGeometryBuilder::build(object, options)
  }

  /// Whether the object exports as a skeleton, `CEditableObject::IsSkeleton`: whether it has bones, whatever its
  /// dynamic flag says.
  fn is_skeleton(object: &EditableObjectFile) -> bool {
    object.bones.as_ref().is_some_and(|bones| !bones.is_empty())
  }
}
