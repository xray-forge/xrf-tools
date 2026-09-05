//! The bump half of the textures check, against descriptor trees built from `xrf-material`'s own fixtures.
//!
//! Every texture file here is a placeholder, so the DDS half reports each as unreadable; the assertions read the bump
//! counts and the bump rules, which is what this file pins.

use std::fs;

use xrf_db::{ThmBumpChunk, ThmTextureTypeChunk};
use xrf_material::fixtures::{ThmFixture, ThmFixtureTree};

use crate::project::textures::verify_textures_result::GamedataTexturesVerificationResult;
use crate::{
  Finding, GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions,
  GamedataVerificationRule,
};

const BASE: &str = "act\\act_stalker";
const BUMP: &str = "act\\act_stalker_bump";
const COMPANION: &str = "act\\act_stalker_bump#";

/// A gamedata root over the fixture tree, which needs only a `system.ltx` to be a project.
fn project(tree: &ThmFixtureTree) -> GamedataProject {
  let configs = tree.root().join("configs");

  fs::create_dir_all(&configs).expect("configs directory");
  fs::write(configs.join("system.ltx"), "").expect("system.ltx written");

  GamedataProject::open(&GamedataProjectReadOptions {
    root: tree.root().to_path_buf(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .expect("project opens")
}

fn verify(tree: &ThmFixtureTree) -> GamedataTexturesVerificationResult {
  project(tree)
    .verify_textures(&GamedataProjectVerifyOptions::default())
    .expect("textures verified")
}

fn rules_of(result: &GamedataTexturesVerificationResult) -> Vec<String> {
  result
    .get_findings()
    .iter()
    .filter(|finding| finding.rule_id().to_string().starts_with("textures.bump"))
    .map(|finding: &Finding| finding.rule_id().to_string())
    .collect()
}

#[test]
fn a_pair_that_resolves_counts_as_a_resolved_bump() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("gamedata_resolved")
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, BUMP));
  let result: GamedataTexturesVerificationResult = verify(&tree);

  assert_eq!(result.texture_bumps.checked_bumps_count, 1);
  assert_eq!(result.texture_bumps.unresolved_bumps_count, 0);
  assert_eq!(result.texture_bumps.invalid_bump_declarations_count, 0);
  assert!(rules_of(&result).is_empty(), "{:?}", rules_of(&result));
}

#[test]
fn a_missing_companion_is_its_own_rule_and_its_own_count() {
  // The case the previous check never saw: the bump exists, the # does not, and the engine draws a dummy companion.
  // Counted apart from a missing bump, because only strict fails on it.
  let tree: ThmFixtureTree = ThmFixtureTree::new("gamedata_companion")
    .with_engine_dummies()
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, BUMP));
  let result: GamedataTexturesVerificationResult = verify(&tree);

  assert_eq!(result.texture_bumps.checked_bumps_count, 1);
  assert_eq!(
    result.texture_bumps.unresolved_bumps_count, 0,
    "the bump itself is the declared file"
  );
  assert_eq!(result.texture_bumps.missing_companions_count, 1);
  assert_eq!(
    rules_of(&result),
    [GamedataVerificationRule::TexturesBumpCompanion.to_string()]
  );

  let finding: &Finding = result
    .get_findings()
    .iter()
    .find(|finding| finding.rule_id().to_string() == "textures.bump-companion")
    .expect("companion finding");

  assert_eq!(finding.subject(), Some("textures/act/act_stalker.thm"));
  assert!(
    finding.message().contains("'act\\act_stalker_bump#'") && finding.message().contains("'ed\\ed_dummy_bump#'"),
    "names the companion and what the engine binds instead: {}",
    finding.message()
  );
}

#[test]
fn a_missing_bump_reports_both_halves_of_the_pair() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("gamedata_missing")
    .with_engine_dummies()
    .with_texture(BASE)
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, BUMP));
  let result: GamedataTexturesVerificationResult = verify(&tree);

  assert_eq!(result.texture_bumps.checked_bumps_count, 1);
  assert_eq!(result.texture_bumps.unresolved_bumps_count, 1);
  assert_eq!(
    result.texture_bumps.missing_companions_count, 1,
    "each half is counted where it is missing"
  );
  assert_eq!(
    rules_of(&result),
    [
      GamedataVerificationRule::TexturesBump.to_string(),
      GamedataVerificationRule::TexturesBumpCompanion.to_string(),
    ]
  );
}

#[test]
fn a_declaration_the_engine_skips_for_its_type_is_invalid_rather_than_unresolved() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("gamedata_type")
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(
      BASE,
      &ThmFixture::image()
        .with_bump(ThmBumpChunk::MODE_USE, BUMP)
        .with_texture_type(ThmTextureTypeChunk::BUMP_MAP),
    );
  let result: GamedataTexturesVerificationResult = verify(&tree);

  assert_eq!(
    result.texture_bumps.checked_bumps_count, 0,
    "the engine binds nothing for it"
  );
  assert_eq!(result.texture_bumps.invalid_bump_declarations_count, 1);
  assert_eq!(
    rules_of(&result),
    [GamedataVerificationRule::TexturesBumpDeclaration.to_string()]
  );
  assert!(
    result
      .get_failure_message()
      .ends_with("1 bump declarations the engine never reads")
  );
}

#[test]
fn a_used_mode_with_an_empty_name_is_an_invalid_declaration() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("gamedata_empty")
    .with_texture(BASE)
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, ""));
  let result: GamedataTexturesVerificationResult = verify(&tree);

  assert_eq!(result.texture_bumps.invalid_bump_declarations_count, 1);
  assert_eq!(
    rules_of(&result),
    [GamedataVerificationRule::TexturesBumpDeclaration.to_string()]
  );
}

#[test]
fn a_disabled_declaration_and_an_orphan_descriptor_are_checked_and_clean() {
  // The orphan has no texture beside it and is still walked, because `LoadTHM` walks descriptors rather than textures.
  let tree: ThmFixtureTree = ThmFixtureTree::new("gamedata_disabled")
    .with_texture(BASE)
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_NONE, BUMP))
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(
      "act\\act_orphan",
      &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, BUMP),
    );
  let result: GamedataTexturesVerificationResult = verify(&tree);

  assert_eq!(
    result.texture_bumps.checked_bumps_count, 1,
    "the orphan's pair is bound and counted"
  );
  assert_eq!(result.texture_bumps.unresolved_bumps_count, 0);
  assert_eq!(result.texture_bumps.invalid_bump_declarations_count, 0);
  assert!(rules_of(&result).is_empty());
}

#[test]
fn an_unreadable_descriptor_is_reported_under_the_read_rule() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("gamedata_unreadable")
    .with_texture(BASE)
    .with_unreadable_descriptor(BASE);
  let result: GamedataTexturesVerificationResult = verify(&tree);

  assert!(
    result.get_findings().iter().any(|finding| {
      finding.rule_id().to_string() == "textures.read" && finding.subject() == Some("textures/act/act_stalker.thm")
    }),
    "{:?}",
    result
      .get_findings()
      .iter()
      .map(|finding| (finding.rule_id().to_string(), finding.subject().map(str::to_owned)))
      .collect::<Vec<_>>()
  );
}
