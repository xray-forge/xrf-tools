use xrf_chunk::{ChunkDataSource, ChunkReader, ChunkWriter};
use xrf_error::XrfResult;

use crate::chunks::level::ogf_geometry_container_chunk::OgfGeometryContainerChunk;
use crate::chunks::mesh::ogf_swi_data_chunk::OgfSwiDataChunk;

/// The fast geometry a level visual draws for depth and shadows from `level.geomX`, `OGF_FASTPATH`.
#[derive(Debug)]
pub struct OgfFastpathChunk {
  pub geometry: OgfGeometryContainerChunk,
  /// The sliding windows of a progressive visual's fast geometry, absent for a static one.
  pub swi: Option<OgfSwiDataChunk>,
}

impl OgfFastpathChunk {
  /// `OGF_FASTPATH` (`xray-16/src/xrCore/FMesh.hpp`).
  pub const CHUNK_ID: u32 = 22;

  /// Reads the fast path from a reader positioned at its chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the chunk holds no geometry container, which the engine opens unconditionally, or a chunk
  /// it holds does not read whole.
  pub fn read_from_chunk<D: ChunkDataSource>(reader: &mut ChunkReader<D>) -> XrfResult<Self> {
    let chunks: Vec<ChunkReader<D>> = reader.read_children()?;

    Ok(Self {
      geometry: ChunkReader::find_required_by_id(&chunks, OgfGeometryContainerChunk::CHUNK_ID)?.read_xr()?,
      swi: match ChunkReader::find_optional_by_id(&chunks, OgfSwiDataChunk::CHUNK_ID) {
        Some(mut chunk) => Some(chunk.read_xr()?),
        None => None,
      },
    })
  }

  /// Writes the fast path's chunks as xrLC does (`write_ogf_fastpath`): its geometry container, then its windows where
  /// it has them. The caller wraps them in the `OGF_FASTPATH` chunk.
  ///
  /// # Errors
  ///
  /// Returns an error when the writer fails.
  pub fn write(&self, writer: &mut ChunkWriter) -> XrfResult {
    let mut geometry: ChunkWriter = ChunkWriter::new();

    geometry.write_xr(&self.geometry)?;
    geometry.flush_chunk_into(&mut writer.buffer, OgfGeometryContainerChunk::CHUNK_ID)?;

    if let Some(swi) = &self.swi {
      let mut windows: ChunkWriter = ChunkWriter::new();

      windows.write_xr(swi)?;
      windows.flush_chunk_into(&mut writer.buffer, OgfSwiDataChunk::CHUNK_ID)?;
    }

    Ok(())
  }
}
