use serde::Serialize;
use xrf_db::{DetailModel, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_vfs::XrayAssetType;

use crate::plugins::archives::describe::archive_bounds::ArchiveBounds;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// Everything the viewer says about one detail object.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveDetailModel {
  /// The blender it draws through, which names a definition inside `shaders.xr` rather than a file, so it crosses as
  /// text and not as a reference.
  pub shader: String,
  /// The texture it draws with, absent when the object names none.
  pub texture: Option<ArchiveReference>,
  pub min_scale: f32,
  pub max_scale: f32,
  /// Whether the renderer sways it in the wind, which is `DO_NO_WAVING` being clear rather than set.
  pub is_waving: bool,
  /// Bits of the flag word no name here claims.
  pub unnamed_flags: u32,
  pub vertices: usize,
  pub triangles: usize,
  /// The box the mesh occupies as authored, before a slot's own scale applies; absent for a model carrying no mesh.
  pub bounds: Option<ArchiveBounds>,
}

impl ArchiveDetailModel {
  /// Reads the standalone detail object an entry holds, which is what a `.dm` is.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not a detail object this reader can
  /// walk - a mesh the payload does not account for, or an index count that is not whole triangles.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    Ok(Self::of(
      source,
      &DetailModel::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?,
    ))
  }

  /// One object as the viewer reads it, with the texture it names resolved.
  pub fn of(source: &ArchiveDescribeSource, model: &DetailModel) -> Self {
    Self {
      shader: model.shader.clone(),
      texture: (!model.texture.is_empty())
        .then(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &model.texture)),
      min_scale: model.min_scale,
      max_scale: model.max_scale,
      is_waving: model.is_waving(),
      unnamed_flags: model.flags & !DetailModel::NO_WAVING,
      vertices: model.vertices.len(),
      triangles: model.get_triangles_count(),
      bounds: model
        .get_bounds()
        .map(|(minimum, maximum): (Vector3d<f32>, Vector3d<f32>)| ArchiveBounds::of(&minimum, &maximum)),
    }
  }
}
