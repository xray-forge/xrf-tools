use std::fs;
use std::path::Path;

use xrf_chunk::ChunkWriter;
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_spawn::{LevelSpawnFile, LevelSpawnObject, XRayByteOrder};

/// An object of a level's spawn list, its class-dependent rest left opaque.
pub(crate) fn new_level_object(id: u32, section: &str, name: &str) -> LevelSpawnObject {
  LevelSpawnObject {
    id,
    // `M_SPAWN`, which every packet of a list opens with.
    net_action: 1,
    section: String::from(section),
    name: String::from(name),
    script_game_id: 0,
    script_rp: 0xFF,
    position: Vector3d::new(1.0, 2.0, 3.0),
    direction: Vector3d::ZERO,
    payload: vec![7; 12],
  }
}

/// Writes `objects` to `path` as a level's spawn list, answering the bytes written.
pub(crate) fn write_level_spawn_list(path: &Path, objects: Vec<LevelSpawnObject>) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelSpawnFile { objects }.write::<XRayByteOrder>(&mut writer)?;

  let bytes: Vec<u8> = writer.flush_raw_into_buffer()?;

  fs::write(path, &bytes)?;

  Ok(bytes)
}
