use xrf_game_developer::DeveloperLoading;
use xrf_renderer::RenderLoadReport;

use crate::run::game_startup::GameStartup;

#[test]
fn counts_sectors_and_textures_together_towards_the_level_loaded() {
  let loading: DeveloperLoading = GameStartup::create_loading(
    "zaton",
    Some(RenderLoadReport {
      sectors: 30,
      sectors_total: 100,
      textures: 470,
      textures_total: 900,
      ..RenderLoadReport::default()
    }),
  );

  assert_eq!(loading.title, "Loading zaton");
  assert!((loading.progress - 0.5).abs() < 1e-6);
  assert_eq!(loading.detail, "Sectors 30 of 100, textures 470 of 900");
}

#[test]
fn starts_empty_before_the_renderer_reports() {
  let loading: DeveloperLoading = GameStartup::create_loading("zaton", None);

  assert!(loading.progress.abs() < 1e-6);
  assert_eq!(loading.detail, "Starting the renderer");
}
