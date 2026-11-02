use serde::Serialize;
use xrf_gamemtl::{GameMtlFile, GameMtlVersion};

/// What `gamemtl info` read out of a game material library.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameMtlInfoReport {
  version: u16,
  game: GameMtlVersion,
  materials: usize,
  dynamic_materials: usize,
  pairs: usize,
  inheriting_pairs: usize,
}

impl GameMtlInfoReport {
  pub fn new(file: &GameMtlFile) -> Self {
    Self {
      version: file.version,
      game: file.get_game_version(),
      materials: file.materials.len(),
      dynamic_materials: file.get_dynamic_materials_count(),
      pairs: file.pairs.len(),
      inheriting_pairs: file.get_inheriting_pairs_count(),
    }
  }
}
