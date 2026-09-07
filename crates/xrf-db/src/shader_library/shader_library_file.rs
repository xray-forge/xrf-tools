use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, XRayByteOrder, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::shader_library::shader_blender::ShaderBlender;

/// The compiled blender library, `shaders.xr`.
///
/// The renderer resolves the shader name an OGF texture chunk, a level surface or a config declares against these
/// definitions when it creates the shader (`Layers/xrRender/ResourceManager.cpp`), so the library is where a
/// surface's render states come from - alpha testing and blending included. Nothing in the mesh or in the texture
/// says whether alpha is read: the blender does.
///
/// Only the blender chunk is read. The file also carries the shader script list, the constant table and the matrix
/// table, none of which name a surface, and none of which are written back: this reader has no writer, so a library
/// is never rewritten with three of its four chunks missing.
#[derive(Debug, Default)]
pub struct ShaderLibraryFile {
  blenders: HashMap<String, ShaderBlender>,
}

impl ShaderLibraryFile {
  pub const BLENDERS_CHUNK_ID: u32 = 2;

  pub fn read_from_path<P: AsRef<Path>>(path: P) -> XrfResult<Self> {
    Self::read_from_file(File::open(path.as_ref()).map_err(|error| {
      XrfError::new_not_found_error(format!(
        "Shader library was not read: {}, error: {error}",
        format_path(path.as_ref())
      ))
    })?)
  }

  pub fn read_from_file(file: File) -> XrfResult<Self> {
    Self::read_from_chunk(&mut ChunkReader::from_file(file)?)
  }

  pub fn read_from_chunk<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut blenders: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::BLENDERS_CHUNK_ID)?;

    Self::read_blenders(&mut blenders)
  }

  /// The blender a shader name resolves to, or `None` for a name the library does not define.
  ///
  /// An absent name is what the engine reports as `! Shader '%s' not found in library` before falling back to the
  /// default shader (`ResourceManager.cpp`), so it is an answer rather than a failure.
  pub fn find_blender(&self, name: &str) -> Option<&ShaderBlender> {
    self.blenders.get(name)
  }

  pub fn contains_blender(&self, name: &str) -> bool {
    self.blenders.contains_key(name)
  }

  pub fn blenders_count(&self) -> usize {
    self.blenders.len()
  }

  pub fn blenders(&self) -> impl Iterator<Item = &ShaderBlender> {
    self.blenders.values()
  }

  /// Reads every blender chunk, refusing a library that defines one name twice.
  ///
  /// The engine refuses it too, with `R_ASSERT2(I.second, "shader.xr - found duplicate name!!!")`
  /// (`ResourceManager_Loader.cpp`): a duplicate means the second definition is unreachable, and which of the two
  /// a surface gets would otherwise depend on iteration order here.
  fn read_blenders<D: ChunkDataSource>(blenders: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = blenders.read_children()?;
    let mut library: HashMap<String, ShaderBlender> = HashMap::with_capacity(chunks.len());

    for mut chunk in chunks {
      let blender: ShaderBlender = ShaderBlender::read::<XRayByteOrder, D>(&mut chunk)?;

      chunk.assert_read("Expect all data to be read from shader blender")?;

      if library.contains_key(&blender.name) {
        return Err(XrfError::new_invalid_error(format!(
          "Shader library contains duplicate blender '{}'",
          blender.name
        )));
      }

      library.insert(blender.name.clone(), blender);
    }

    Ok(Self { blenders: library })
  }
}
