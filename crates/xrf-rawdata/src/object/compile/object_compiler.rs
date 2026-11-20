use std::time::{SystemTime, UNIX_EPOCH};

use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{OutputFiles, StagedFile, format_path};

use crate::object::compile::detail::detail_model_builder::DetailModelBuilder;
use crate::object::compile::object_compile::ObjectCompile;
use crate::object::compile::object_compile_options::ObjectCompileOptions;
use crate::object::compile::object_compile_report::ObjectCompileReport;
use crate::object::compile::object_compile_request::ObjectCompileRequest;
use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::rigid::rigid_geometry::RigidGeometry;
use crate::object::compile::rigid::rigid_geometry_builder::RigidGeometryBuilder;
use crate::object::compile::rigid::rigid_ogf_writer::RigidOgfWriter;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_geometry_builder::SkeletonGeometryBuilder;
use crate::object::compile::skeleton::skeleton_ogf_writer::SkeletonOgfWriter;
use crate::object::detail_parameters::DetailParameters;
use crate::provenance::object_provenance::ObjectProvenance;

/// Turns an editable object into what the Actor Editor's `Export OGF` writes, for the engine to load: see the crate's
/// documentation for what matches the editor's output and what only matches what the engine draws.
pub struct ObjectCompiler;

impl ObjectCompiler {
  /// Compiles the `.object` `request` names into its destination: a detail model where it ends in `.dm`, with the
  /// sidecar's flags and scale range or the editor's defaults, else the OGF recording the source, `tool` and the time.
  /// An existing destination is refused before the object is read unless the request is forced.
  ///
  /// # Errors
  ///
  /// Returns an error when the destination exists unforced, the object or its sidecar does not read, the compile fails
  /// (see [`Self::compile`] and [`Self::compile_detail`]), or the file is not written.
  pub fn compile_to_path(
    request: &ObjectCompileRequest,
    tool: &str,
    tool_version: &str,
  ) -> XrfResult<ObjectCompileReport> {
    OutputFiles::of_path(&request.destination).assert_free(request.is_forced)?;

    let object: EditableObjectFile = EditableObjectFile::read_from_path::<XRayByteOrder, _>(&request.source)?;
    let compiled: ObjectCompile = if request.is_detail() {
      let parameters: DetailParameters = ObjectProvenance::read_beside(&request.source)?
        .and_then(|provenance| provenance.detail)
        .unwrap_or_default();

      Self::compile_detail(&object, &parameters)?
    } else {
      Self::compile_with(
        &object,
        &ObjectCompileSource {
          source_file: format_path(&request.source).to_string(),
          convertor: format!("{tool} {tool_version}"),
          built_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs()),
        },
        &request.get_options()?,
      )?
    };

    StagedFile::write_creating_directories(&request.destination, &compiled.bytes).map_err(|error| {
      XrfError::new_io_error(
        format!("{} was not written: {error}", format_path(&request.destination)),
        error.kind(),
      )
    })?;

    Ok(ObjectCompileReport {
      source: format_path(&request.source).to_string(),
      destination: format_path(&request.destination).to_string(),
      children: compiled.children,
      vertices: compiled.vertices,
      faces: compiled.faces,
      dropped_faces: compiled.dropped_faces,
      bones: object.bones.as_ref().map_or(0, Vec::len),
      bytes: compiled.bytes.len(),
      tangents: compiled.tangents,
    })
  }

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
        bytes: Self::write_skeleton(object, &geometry, source)?,
        children: geometry.splits.len(),
        vertices: geometry.splits.iter().map(|split| split.vertices.len()).sum(),
        faces: geometry.splits.iter().map(|split| split.faces.len()).sum(),
        dropped_faces: geometry.dropped_faces,
        tangents: Some(options.tangents),
      })
    } else {
      let geometry: RigidGeometry = RigidGeometryBuilder::build(object)?;

      Ok(ObjectCompile {
        bytes: RigidOgfWriter::write::<XRayByteOrder>(object, &geometry, source)?,
        children: geometry.splits.len(),
        vertices: geometry.splits.iter().map(|split| split.vertices.len()).sum(),
        faces: geometry.splits.iter().map(|split| split.faces.len()).sum(),
        dropped_faces: geometry.dropped_faces,
        tangents: None,
      })
    }
  }

  /// The detail model (`.dm`) of an object of one surface, as the Level Editor's `EDetail` exports it, with
  /// `parameters` for the flags and scale range the object does not hold.
  ///
  /// # Errors
  ///
  /// Returns an error for an object of other than one surface or of no mesh, a corner whose reference names no texture
  /// coordinate first, more than 65,535 vertices, or a mesh that does not hold together.
  pub fn compile_detail(object: &EditableObjectFile, parameters: &DetailParameters) -> XrfResult<ObjectCompile> {
    DetailModelBuilder::build::<XRayByteOrder>(object, parameters)
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
