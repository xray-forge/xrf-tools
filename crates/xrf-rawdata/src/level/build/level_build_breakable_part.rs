use byteorder::ByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;

use crate::object::compile::object_compile_source::ObjectCompileSource;
use crate::object::compile::skeleton::skeleton_geometry::SkeletonGeometry;
use crate::object::compile::skeleton::skeleton_ogf_writer::SkeletonOgfWriter;

/// One part's rigid skeleton, kept unwritten so that it takes the byte order of the level's other files.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LevelBuildBreakablePart {
  /// Its path in the level's directory, `meshes/brkbl#<n>.ogf`.
  pub(crate) path: String,
  pub(super) object: EditableObjectFile,
  pub(super) geometry: SkeletonGeometry,
}

impl LevelBuildBreakablePart {
  /// The part's OGF in `T`'s byte order, with a description of nothing (`ogf_desc`'s defaults).
  ///
  /// # Errors
  ///
  /// Returns an error when the skeleton does not write.
  pub(crate) fn write<T: ByteOrder>(&self) -> XrfResult<Vec<u8>> {
    SkeletonOgfWriter::write::<T>(
      &self.object,
      &self.geometry,
      &ObjectCompileSource {
        source_file: String::new(),
        convertor: String::new(),
        built_at: 0,
      },
    )
  }
}
