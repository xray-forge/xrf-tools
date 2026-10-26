use xrf_chunk::{ChunkReadWrite, ChunkReader, ChunkWriter, InMemoryChunkDataSource, XRayByteOrder};
use xrf_error::XrfResult;

use crate::chunks::ogf_description_chunk::OgfDescriptionChunk;
use crate::chunks::ogf_description_time_width::OgfDescriptionTimeWidth;
use crate::chunks::ogf_texture_chunk::OgfTextureChunk;

/// A description laid out with times `width` bytes wide: the source file, then the convertor, creator and editor, each
/// followed by its time.
fn description_bytes(width: usize) -> Vec<u8> {
  let mut bytes: Vec<u8> = b"F:\\objects\\bandit.object\0".to_vec();

  for (name, time) in [
    ("\\\\DESKTOP\\Dux", 0u64),
    ("\\\\DIEHARD", 1_206_007_999),
    ("Mr.DieHard", 7),
  ] {
    bytes.extend_from_slice(name.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&time.to_le_bytes()[..width]);
  }

  bytes
}

fn read_description(bytes: &[u8]) -> XrfResult<OgfDescriptionChunk> {
  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(bytes)?;

  OgfDescriptionChunk::read::<XRayByteOrder, _>(&mut reader)
}

#[test]
fn reads_a_description_in_whichever_time_width_it_was_written() -> XrfResult {
  for (width, time_width) in [(4, OgfDescriptionTimeWidth::Narrow), (8, OgfDescriptionTimeWidth::Wide)] {
    let bytes: Vec<u8> = description_bytes(width);
    let description: OgfDescriptionChunk = read_description(&bytes)?;

    assert_eq!(description.time_width, time_width);
    assert_eq!(description.source_file, "F:\\objects\\bandit.object");
    assert_eq!(description.created_at, 1_206_007_999);
    assert_eq!(description.editor, "Mr.DieHard");

    let mut writer: ChunkWriter = ChunkWriter::new();

    description.write::<XRayByteOrder>(&mut writer)?;

    assert_eq!(
      writer.buffer, bytes,
      "Expect the {time_width:?} description to write back as read"
    );
  }

  Ok(())
}

#[test]
fn refuses_a_description_neither_width_reads_whole() -> XrfResult {
  let mut bytes: Vec<u8> = description_bytes(4);

  bytes.push(1);

  let error: String = read_description(&bytes)
    .expect_err("a trailing byte fits neither width")
    .to_string();

  assert!(
    error.contains("neither 32-bit nor 64-bit times"),
    "Unexpected error: {error}"
  );

  Ok(())
}

#[test]
fn keeps_the_bytes_a_texture_chunk_carries_past_its_two_names() -> XrfResult {
  // `rat_tail_normal.ogf`'s shape: two names, then padding the engine never reads.
  let bytes: &[u8] = b"act\\act_norma\0models\\artefact2\0\0\0\0\0\0\0\0\0\0\0\0\0";
  let mut reader: ChunkReader<InMemoryChunkDataSource> = ChunkReader::from_bytes(bytes)?;
  let texture: OgfTextureChunk = OgfTextureChunk::read::<XRayByteOrder, _>(&mut reader)?;

  assert_eq!(texture.texture_name, "act\\act_norma");
  assert_eq!(texture.shader_name, "models\\artefact2");
  assert_eq!(texture.trailing, vec![0; 12]);

  let mut writer: ChunkWriter = ChunkWriter::new();

  texture.write::<XRayByteOrder>(&mut writer)?;

  assert_eq!(writer.buffer, bytes);

  Ok(())
}
