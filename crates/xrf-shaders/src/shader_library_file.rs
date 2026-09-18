use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use xrf_chunk::{ChunkDataSource, ChunkReadWrite, ChunkReader, XRayByteOrder, find_required_chunk_by_id};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::shader_blender::ShaderBlender;

/// The compiled blender library, `shaders.xr`.
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

  /// Reads from bytes already in hand, which is how an archived library arrives: a volume holds no file to open.
  pub fn read_from_bytes(bytes: Vec<u8>) -> XrfResult<Self> {
    Self::read_from_chunk(&mut ChunkReader::from_vec(bytes)?)
  }

  pub fn read_from_chunk<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;
    let mut blenders: ChunkReader<D> = find_required_chunk_by_id(&chunks, Self::BLENDERS_CHUNK_ID)?;

    Self::read_blenders(&mut blenders)
  }

  /// The blender a shader name resolves to, or `None` for a name the library does not define.
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
