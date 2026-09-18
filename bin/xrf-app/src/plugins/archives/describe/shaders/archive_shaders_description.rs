use serde::Serialize;
use xrf_error::XrfResult;
use xrf_shaders::ShaderLibraryFile;

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::shaders::archive_shaders_blender::ArchiveShadersBlender;
use crate::plugins::archives::describe::shaders::archive_shaders_library::ArchiveShadersLibrary;

/// Everything the viewer says about the blender library.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveShadersDescription {
  pub library: ArchiveShadersLibrary,
  pub blenders: Vec<ArchiveShadersBlender>,
}

impl ArchiveShadersDescription {
  /// Reads the library an entry holds and resolves the textures its blenders bind.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, when they carry no blender chunk, or when the library
  /// defines one shader name twice - which the engine refuses too.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: ShaderLibraryFile = ShaderLibraryFile::read_from_bytes(source.read_bytes(name)?)?;

    let blenders: Vec<ArchiveShadersBlender> = ArchiveShadersBlender::of_all(source, file.blenders());

    Ok(Self {
      library: ArchiveShadersLibrary::of(&blenders),
      blenders,
    })
  }
}
