use std::sync::Arc;

use xrf_game_developer::DeveloperLoading;
use xrf_renderer::RenderLoadReport;

use crate::load::game_load_progress::GameLoadProgress;
use crate::load::game_load_stage::GameLoadStage;
use crate::load::game_loading::GameLoading;

#[test]
fn counts_each_stage_an_equal_share_and_a_running_one_its_fraction() {
  let progress: GameLoadProgress = GameLoadProgress::new(&[
    GameLoadStage::Tree,
    GameLoadStage::Level,
    GameLoadStage::Physics,
    GameLoadStage::Streaming,
  ]);

  assert!(progress.get_progress().abs() < 1e-6);

  progress.run(GameLoadStage::Tree, || ());
  progress.begin(GameLoadStage::Level);
  progress.advance(GameLoadStage::Streaming, 0.5);
  // A stage this load does not take changes nothing.
  progress.finish(GameLoadStage::Renderer);

  assert!((progress.get_progress() - 1.5 / 4.0).abs() < 1e-6);
  assert_eq!(
    progress.list_running(),
    vec![GameLoadStage::Level, GameLoadStage::Streaming]
  );
  assert!(!progress.is_done());

  for stage in [GameLoadStage::Level, GameLoadStage::Physics, GameLoadStage::Streaming] {
    progress.finish(stage);
  }

  assert!(progress.is_done());
  assert!((progress.get_progress() - 1.0).abs() < 1e-6);
}

#[test]
fn names_the_stages_running_and_the_streamings_counts() {
  let progress: Arc<GameLoadProgress> = Arc::new(GameLoadProgress::new(&GameLoadStage::LAUNCH));
  let loading: GameLoading = GameLoading::new("zaton", Arc::clone(&progress));

  progress.begin(GameLoadStage::Physics);
  progress.advance(GameLoadStage::Streaming, 0.5);

  let shown: DeveloperLoading = loading.describe(Some(RenderLoadReport {
    sectors: 30,
    sectors_total: 100,
    textures: 470,
    textures_total: 900,
    ..RenderLoadReport::default()
  }));

  assert_eq!(shown.title, "Loading zaton");
  assert!((shown.progress - 0.5 / 6.0).abs() < 1e-6);
  assert_eq!(
    shown.detail,
    "Building the level's collision · Streaming the level: sectors 30 of 100, textures 470 of 900"
  );
}
