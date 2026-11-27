use std::sync::Arc;

use xrf_gamemtl::GameMtlFile;
use xrf_level_source::LevelAudio;

use crate::host::game_configs::GameConfigs;
use crate::host::game_tree::GameTree;

/// What the run reads of its game tree once and keeps for every game and level played on it: the tree mounted, its
/// configs (`system.ltx` resolved once, every other config read through the same store), its game materials, and the
/// audio server every shown level plays through. It outlives each game and level, as the window and the renderer
/// (`GameShell`) do on the main thread.
pub struct GameHost {
  tree: GameTree,
  configs: Arc<GameConfigs>,
  materials: Arc<GameMtlFile>,
  audio: LevelAudio,
}

impl GameHost {
  /// The host of `tree`, its `configs` and `materials` read, its levels heard through `audio`.
  pub fn new(tree: GameTree, configs: GameConfigs, materials: GameMtlFile, audio: LevelAudio) -> Self {
    Self {
      tree,
      configs: Arc::new(configs),
      materials: Arc::new(materials),
      audio,
    }
  }

  pub fn get_tree(&self) -> &GameTree {
    &self.tree
  }

  pub fn get_configs(&self) -> &Arc<GameConfigs> {
    &self.configs
  }

  pub fn get_materials(&self) -> &Arc<GameMtlFile> {
    &self.materials
  }

  /// Where a shown level is heard: a clone shares the one server.
  pub fn get_audio(&self) -> &LevelAudio {
    &self.audio
  }
}
