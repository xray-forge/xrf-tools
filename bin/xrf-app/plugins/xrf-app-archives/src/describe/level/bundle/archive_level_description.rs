use serde::Serialize;
use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_level::{LevelFile, LevelShaderEntry};
use xrf_shaders::ShaderLibraryFile;
use xrf_vfs::XrayAssetType;

use crate::describe::description::ArchiveReference;
use crate::describe::level::bundle::ArchiveLevelBundle;
use crate::describe::level::bundle::ArchiveLevelSurface;
use crate::describe::source::ArchiveDescribeSource;

/// Everything the viewer says about a compiled level bundle.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelDescription {
  pub bundle: ArchiveLevelBundle,
  pub surfaces: Vec<ArchiveLevelSurface>,
}

impl ArchiveLevelDescription {
  /// Reads the bundle an entry holds, resolving its textures and asking its shader names of the library beside it.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not a level bundle - which a file named
  /// `level` that is something else will be.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: LevelFile = LevelFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;

    // The engine loads the blender library from the root of whatever tree is open.
    let entry: Option<String> = source.find_entry(XrayAssetType::SHADER_LIBRARY_PATH);
    let library: Option<ShaderLibraryFile> = entry.as_deref().and_then(|entry| Self::read_library(source, entry));
    let entries: &[LevelShaderEntry] = file.shaders.as_ref().map_or(&[], |shaders| shaders.entries.as_slice());

    let surfaces: Vec<ArchiveLevelSurface> = ArchiveLevelSurface::of_all(source, entries, library.as_ref());

    Ok(Self {
      bundle: ArchiveLevelBundle::of(
        file.header.xrlc_version,
        file.header.xrlc_quality,
        &surfaces,
        entry.map(|_| ArchiveReference::of_path(source, XrayAssetType::SHADER_LIBRARY_PATH)),
        file.shaders.is_some(),
      ),
      surfaces,
    })
  }

  /// The blender library of the subject being browsed, or `None` when it holds none or cannot read the one it holds.
  fn read_library(source: &ArchiveDescribeSource, entry: &str) -> Option<ShaderLibraryFile> {
    ShaderLibraryFile::read_from_bytes(source.read_bytes(entry).ok()?).ok()
  }
}
