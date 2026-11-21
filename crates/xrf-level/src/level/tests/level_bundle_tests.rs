use xrf_chunk::{ChunkReader, ChunkWriter, InMemoryChunkDataSource};
use xrf_error::XrfResult;

use crate::level::level_bundle::LevelBundle;
use crate::level::level_header_chunk::LevelHeaderChunk;
use crate::level::sectors::level_sector::LevelSector;
use crate::level::sectors::level_sectors_chunk::LevelSectorsChunk;
use crate::level::shaders::level_shader_entry::LevelShaderEntry;
use crate::level::shaders::level_shaders_chunk::LevelShadersChunk;

fn bundle() -> LevelBundle {
  LevelBundle {
    header: LevelHeaderChunk {
      xrlc_version: 14,
      xrlc_quality: 0,
    },
    portals: None,
    lights: None,
    glows: None,
    visuals: None,
    shaders: Some(LevelShadersChunk {
      entries: vec![LevelShaderEntry::Empty, LevelShaderEntry::parse("default/brick")],
    }),
    sectors: Some(vec![
      LevelSector {
        portals: vec![0, 1],
        root: 7,
      },
      LevelSector {
        portals: vec![],
        root: 9,
      },
    ]),
  }
}

fn write(bundle: &LevelBundle) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  bundle.write(&mut writer)?;

  Ok(writer.into_bytes())
}

#[test]
fn test_a_written_bundle_reads_back_and_writes_again_byte_for_byte() -> XrfResult {
  let bytes: Vec<u8> = write(&bundle())?;
  let read: LevelBundle = LevelBundle::read_from_bytes(bytes.clone())?;

  assert_eq!(read.header, bundle().header);
  assert_eq!(read.shaders, bundle().shaders);
  assert_eq!(read.sectors, bundle().sectors);
  assert!(
    read.portals.is_none() && read.visuals.is_none(),
    "an absent chunk stays absent"
  );
  assert_eq!(write(&read)?, bytes);

  Ok(())
}

#[test]
fn test_chunks_go_in_xrlc_order_and_a_sector_writes_its_root_first() -> XrfResult {
  let chunks: Vec<ChunkReader<InMemoryChunkDataSource>> = ChunkReader::from_vec(write(&bundle())?)?.read_children()?;

  assert_eq!(
    chunks.iter().map(|chunk| chunk.id).collect::<Vec<u32>>(),
    vec![
      LevelHeaderChunk::CHUNK_ID,
      LevelShadersChunk::CHUNK_ID,
      LevelSectorsChunk::CHUNK_ID
    ]
  );

  let sector: Vec<ChunkReader<InMemoryChunkDataSource>> =
    chunks[2].clone().read_children()?[0].clone().read_children()?;

  assert_eq!(
    sector.iter().map(|chunk| chunk.id).collect::<Vec<u32>>(),
    vec![LevelSector::ROOT_CHUNK_ID, LevelSector::PORTALS_CHUNK_ID]
  );

  Ok(())
}
