use serde::Serialize;
use xrf_db::{LevelFile, LevelShaderEntry, ShaderLibraryFile, XRayByteOrder};
use xrf_error::XrfResult;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;
use crate::plugins::archives::describe::level::archive_level_bundle::ArchiveLevelBundle;
use crate::plugins::archives::describe::level::archive_level_surface::ArchiveLevelSurface;

/// Where the engine loads the blender library from, which is the root of whatever tree is open.
const SHADER_LIBRARY: &str = "shaders.xr";

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

    let entry: Option<String> = source.find_entry(SHADER_LIBRARY);
    let library: Option<ShaderLibraryFile> = entry.as_deref().and_then(|entry| Self::read_library(source, entry));
    let entries: &[LevelShaderEntry] = file.shaders.as_ref().map_or(&[], |shaders| shaders.entries.as_slice());

    let surfaces: Vec<ArchiveLevelSurface> = ArchiveLevelSurface::of_all(source, entries, library.as_ref());

    Ok(Self {
      bundle: ArchiveLevelBundle::of(
        file.header.xrlc_version,
        file.header.xrlc_quality,
        &surfaces,
        entry.map(|_| ArchiveReference::of_path(source, SHADER_LIBRARY)),
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
