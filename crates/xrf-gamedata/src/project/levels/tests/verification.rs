use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::project::levels::level_engine_constants::{AI_CURRENT_VERSION, LEVEL_PRODUCTION_VERSION};
use crate::project::levels::tests::fixtures::*;
use crate::project::levels::verify_levels_result::GamedataLevelsVerificationResult;
use crate::{
  GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions,
  GamedataVerificationStatus,
};

#[test]
fn passes_on_a_consistent_level_tree() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new().verify();

  assert_eq!(
    result.get_status(),
    GamedataVerificationStatus::Passed,
    "findings: {:?}",
    result
      .get_findings()
      .iter()
      .map(|finding| format!("{}: {}", finding.rule_id(), finding.message()))
      .collect::<Vec<_>>()
  );
  assert_eq!(result.roster_levels_count, 1);
  assert_eq!(result.checked_levels_count, 1);
  assert_eq!(
    result.get_failure_message(),
    "1/1 level bundles valid; 2/2 level shader references valid"
  );
}

#[test]
fn skips_verification_without_any_graph_spawn_file() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new().with_spawn(None).verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Skipped);
  assert!(result.get_findings().is_empty());
}

#[test]
fn reports_graph_level_without_a_bundle() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new().with_bundles(Vec::new()).verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.missing-bundle");
}

#[test]
fn reports_bundle_unreachable_from_the_graph() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton"),
      LevelBundleFixture::valid("mp_pool"),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.orphan-bundle");
}

#[test]
fn reports_graph_level_without_a_declared_map() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new().with_declared_maps(Vec::new()).verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.undeclared-map");
}

#[test]
fn accepts_declared_map_without_a_bundle() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_declared_maps(vec![String::from("zaton"), String::from("mp_pool")])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

#[test]
fn reports_duplicate_level_in_a_single_graph() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_spawn(Some(spawn_graph_bytes(
      &[("zaton", 108, ZATON_GUID), ("zaton", 109, ZATON_GUID)],
      vec![
        cross_table(ZATON_GUID, GRAPH_GUID, AI_NODE_COUNT),
        cross_table(ZATON_GUID, GRAPH_GUID, AI_NODE_COUNT),
      ],
    )))
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.graph-duplicate");
}

#[test]
fn reports_missing_required_bundle_file() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![LevelBundleFixture::valid("zaton").without("level.geom")])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.missing-file");
}

#[test]
fn requires_ai_map_only_for_levels_the_graph_declares() {
  let with_graph_level: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![LevelBundleFixture::valid("zaton").without("level.ai")])
    .verify();

  assert_only_rule(&with_graph_level, "levels.missing-file");

  // The same bundle without an AI-map is only an orphan when the graph does not declare it.
  let without_graph_level: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton"),
      LevelBundleFixture::valid("mp_pool").without("level.ai"),
    ])
    .verify();

  assert_only_rule(&without_graph_level, "levels.orphan-bundle");
}

#[test]
fn reports_empty_required_bundle_file() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![LevelBundleFixture::valid("zaton").with("level.geom", Vec::new())])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.file-empty");
}

#[test]
fn reports_detail_description_without_its_texture() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![LevelBundleFixture::valid("zaton").with("level.details", vec![1])])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.details-pair");
}

#[test]
fn accepts_detail_description_together_with_its_texture() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton")
        .with("level.details", vec![1])
        .with("build_details.dds", vec![1]),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

#[test]
fn reports_unreadable_level_configuration() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton").with("level.ltx", b"[duplicate]\n\n[duplicate]\n".to_vec()),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.ltx-read");
}

#[test]
fn reports_missing_level_map_texture() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new().without_texture("map\\map_zaton").verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.map-texture");
}

#[test]
fn reports_incompatible_level_version() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton").with("level", level_file_bytes(13, true, &["shader\\one/level\\ground"])),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.header-version");
}

#[test]
fn reports_level_without_a_shaders_chunk() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton").with("level", level_file_bytes(LEVEL_PRODUCTION_VERSION, false, &[])),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.shaders-chunk");
}

#[test]
fn reports_truncated_level_file() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![LevelBundleFixture::valid("zaton").with("level", vec![1, 2, 3])])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.file-truncated");
}

#[test]
fn reports_incompatible_collision_form_version() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton").with("level.cform", cform_bytes(3)),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.cform-version");
}

#[test]
fn reports_truncated_collision_form_header() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton").with("level.cform", vec![1, 2, 3, 4]),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.file-truncated");
}

#[test]
fn reports_unsupported_ai_map_version() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton").with("level.ai", ai_map_bytes(7, AI_NODE_COUNT, ZATON_GUID)),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.ai-version");
}

#[test]
fn reports_ai_map_that_does_not_match_the_graph_level() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("zaton").with("level.ai", ai_map_bytes(AI_CURRENT_VERSION, AI_NODE_COUNT, OTHER_GUID)),
    ])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_eq!(
    rule_ids(&result),
    BTreeSet::from([String::from("levels.ai-guid"), String::from("levels.level-guid"),])
  );
}

#[test]
fn reports_cross_table_that_does_not_match_the_game_graph() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_spawn(Some(spawn_graph_bytes(
      &[("zaton", 108, ZATON_GUID)],
      vec![cross_table(ZATON_GUID, OTHER_GUID, AI_NODE_COUNT)],
    )))
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.graph-guid");
}

#[test]
fn reports_ai_node_count_that_disagrees_with_the_cross_table() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_spawn(Some(spawn_graph_bytes(
      &[("zaton", 108, ZATON_GUID)],
      vec![cross_table(ZATON_GUID, GRAPH_GUID, AI_NODE_COUNT + 1)],
    )))
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.ai-node-count");
}

#[test]
fn addresses_cross_tables_by_ascending_level_id_rank() {
  // Level ids are sparse and unordered in shipped graphs, so the cross table of the level with the
  // lowest id is the first one regardless of the order levels appear in the file.
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![
      LevelBundleFixture::valid("jupiter")
        .with("level.ai", ai_map_bytes(AI_CURRENT_VERSION, AI_NODE_COUNT, OTHER_GUID)),
      LevelBundleFixture::valid("zaton"),
    ])
    .with_declared_maps(vec![String::from("zaton"), String::from("jupiter")])
    .with_spawn(Some(spawn_graph_bytes(
      &[("jupiter", 115, OTHER_GUID), ("zaton", 108, ZATON_GUID)],
      vec![
        cross_table(ZATON_GUID, GRAPH_GUID, AI_NODE_COUNT),
        cross_table(OTHER_GUID, GRAPH_GUID, AI_NODE_COUNT),
      ],
    )))
    .verify();

  assert_eq!(
    result.get_status(),
    GamedataVerificationStatus::Passed,
    "findings: {:?}",
    result
      .get_findings()
      .iter()
      .map(|finding| format!("{}: {}", finding.rule_id(), finding.message()))
      .collect::<Vec<_>>()
  );
}

#[test]
fn reports_level_texture_reference_that_does_not_resolve() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![LevelBundleFixture::valid("zaton").with(
      "level",
      level_file_bytes(LEVEL_PRODUCTION_VERSION, true, &["shader\\one/level\\missing"]),
    )])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.texture-reference");
}

#[test]
fn reports_shader_table_entry_without_a_delimiter() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![LevelBundleFixture::valid("zaton").with(
      "level",
      level_file_bytes(LEVEL_PRODUCTION_VERSION, true, &["no_delimiter"]),
    )])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Failed);
  assert_only_rule(&result, "levels.shader-reference");
}

#[test]
fn skips_empty_shader_table_entries_like_the_renderer() {
  let result: GamedataLevelsVerificationResult = GamedataFixture::new()
    .with_bundles(vec![LevelBundleFixture::valid("zaton").with(
      "level",
      level_file_bytes(LEVEL_PRODUCTION_VERSION, true, &["", "shader\\one/level\\ground"]),
    )])
    .verify();

  assert_eq!(result.get_status(), GamedataVerificationStatus::Passed);
}

#[test]
fn fails_the_check_when_the_game_graph_cannot_be_read() {
  let unique: u64 = NEXT_TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("levels/unreadable-graph-{unique}"));

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(root.join("configs").join("$scheme")).unwrap();
  fs::write(root.join("configs").join("system.ltx"), "").unwrap();
  fs::create_dir_all(root.join("spawns")).unwrap();
  fs::write(root.join("spawns").join("all.spawn"), [1, 2, 3, 4, 5, 6, 7, 8]).unwrap();

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .unwrap();

  assert!(
    project.verify_levels(&GamedataProjectVerifyOptions::default()).is_err(),
    "Expected an unreadable game graph to fail the check instead of passing it"
  );
}
