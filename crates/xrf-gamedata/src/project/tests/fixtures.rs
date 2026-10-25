use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial, GameMtlPair};

/// The bytes of a game material library, as a project holds it.
pub(crate) fn game_materials_bytes(library: &GameMtlFile) -> Vec<u8> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  library.write::<XRayByteOrder>(&mut writer).unwrap();

  writer.flush_raw_into_buffer().unwrap()
}

/// A library holding every material the engine looks up by name, a static ground material numbered 5 and a dynamic
/// barrel material numbered 6, and the pair the two make.
pub(crate) fn engine_game_materials() -> GameMtlFile {
  let mut default_object: GameMtlMaterial = GameMtlMaterial::new(1, "default_object");
  let mut barrel: GameMtlMaterial = GameMtlMaterial::new(6, "objects\\barrel");

  default_object.flags = GameMtlMaterial::FLAG_DYNAMIC;
  barrel.flags = GameMtlMaterial::FLAG_DYNAMIC;

  GameMtlFile::new(
    vec![
      GameMtlMaterial::new(0, "default"),
      default_object,
      GameMtlMaterial::new(2, "objects\\small_box"),
      GameMtlMaterial::new(3, "objects\\bullet"),
      GameMtlMaterial::new(4, "objects\\knife"),
      GameMtlMaterial::new(5, "materials\\earth"),
      barrel,
    ],
    vec![GameMtlPair::new(0, 5, 6)],
  )
}
