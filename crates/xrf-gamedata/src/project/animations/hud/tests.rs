//! How the player HUD check binds a weapon HUD's motions to the hands, as each engine does, and how it reports them.

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use xrf_engine_target::XrayEngine;
use xrf_test_utils::GeneratedResources;

use crate::project::animations::hud::player_hud_animations_verification_result::GamedataPlayerHudAnimationsVerificationResult;
use crate::project::animations::hud::player_hud_animations_verifier::PlayerHudAnimationsVerifier;
use crate::project::animations::hud::player_hud_motion_requests::PlayerHudMotionRequests;
use crate::project::animations::hud::player_hud_motion_verdict::PlayerHudMotionVerdict;
use crate::project::animations::hud::player_hud_motions::PlayerHudMotions;
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions};

fn motions(names: &[&str]) -> PlayerHudMotions {
  PlayerHudMotions::new(names.iter().map(|name| (*name).to_owned()).collect())
}

fn hands(sets: &[(&str, &[&str])]) -> Vec<(String, PlayerHudMotions)> {
  sets
    .iter()
    .map(|(name, names)| ((*name).to_owned(), motions(names)))
    .collect()
}

#[test]
fn a_motion_plays_by_its_own_name_or_a_numbered_variant() {
  // `player_hud_motion_container::load` binds `name` and `name1` to `name8`, whichever exist, in both engines.
  let hands: PlayerHudMotions = motions(&["idle", "shoot3", "reload9"]);

  assert!(hands.plays("idle"));
  assert!(hands.plays("shoot"), "a numbered variant stands for its base");
  assert!(!hands.plays("reload"), "the engine tries no variant past 8");
  assert!(!hands.plays("aim"));
}

#[test]
fn only_the_extended_engine_falls_back_to_the_idle_motion() {
  let with_idle: PlayerHudMotions = motions(&["idle", PlayerHudMotions::IDLE_FALLBACK]);

  assert!(with_idle.falls_back(XrayEngine::Extended));
  assert!(!with_idle.falls_back(XrayEngine::Vanilla));
  assert!(!motions(&["idle"]).falls_back(XrayEngine::Extended));
}

#[test]
fn one_missing_motion_is_one_finding_however_many_huds_and_hands_it_reaches() {
  // Anomaly's `ak74_idle_moving_w_glm`: a typo several HUD sections inherit, missing from every hands set.
  let mut requests: PlayerHudMotionRequests = PlayerHudMotionRequests::default();

  for hud in ["wpn_ak74_hud", "wpn_ak74m_hud"] {
    requests.add(hud, "anm_idle_moving_g_aim", "ak74_idle_moving_w_glm");
    requests.add(hud, "anm_idle_moving_w_gl_aim", "ak74_idle_moving_w_glm");
    requests.add(hud, "anm_idle", "ak74_idle");
  }

  requests.add("wpn_abakan_hud", "anm_idle_moving_g_aim", "ak74_idle_moving_w_glm");

  let verdict: PlayerHudMotionVerdict = requests.verify(
    &hands(&[
      ("actor_hud_a", &["ak74_idle", "ak74_idle_moving_w_gl"]),
      ("actor_hud_b", &["ak74_idle1", "ak74_idle2"]),
    ]),
    XrayEngine::Vanilla,
  );

  assert_eq!(
    verdict.messages,
    [
      "Motion 'ak74_idle_moving_w_glm', asked for by [wpn_abakan_hud] as anm_idle_moving_g_aim and [wpn_ak74_hud], \
      [wpn_ak74m_hud] as anm_idle_moving_g_aim, anm_idle_moving_w_gl_aim, is missing from all 2 player HUD sections \
      under its own name and as 'ak74_idle_moving_w_glm1' to 'ak74_idle_moving_w_glm8': the engine asserts on it when \
      it binds the HUD to those hands"
    ]
  );
  assert_eq!(
    verdict.invalid_hands,
    BTreeSet::from([String::from("actor_hud_a"), String::from("actor_hud_b")])
  );
}

#[test]
fn the_extended_engine_names_the_idle_motion_it_plays_instead_and_the_hands_that_lack_it() {
  let mut requests: PlayerHudMotionRequests = PlayerHudMotionRequests::default();

  requests.add("wpn_test_hud", "anm_shoot", "shoot");

  let fallback: [(&str, &[&str]); 1] = [("actor_hud", &["idle", PlayerHudMotions::IDLE_FALLBACK])];
  let covered: PlayerHudMotionVerdict = requests.verify(&hands(&fallback), XrayEngine::Extended);
  let partly: PlayerHudMotionVerdict = requests.verify(
    &hands(&[
      ("actor_hud", &["idle", PlayerHudMotions::IDLE_FALLBACK]),
      ("actor_hud_bare", &["idle"]),
    ]),
    XrayEngine::Extended,
  );

  // Still a finding: the motion asked for never plays, which is the author's to fix.
  assert!(
    covered.messages[0]
      .ends_with("the extended engine binds 'hand_idle_doun' in its place, so the motion asked for never plays"),
    "{:?}",
    covered.messages
  );
  assert!(
    partly.messages[0].ends_with(
      "the extended engine binds 'hand_idle_doun' in its place, but binds nothing in [actor_hud_bare], where \
       'hand_idle_doun' is missing too"
    ),
    "{:?}",
    partly.messages
  );
}

#[test]
fn hands_playing_every_motion_or_a_variant_of_it_are_valid() {
  let mut requests: PlayerHudMotionRequests = PlayerHudMotionRequests::default();

  requests.add("wpn_test_hud", "anm_idle", "idle");
  requests.add("wpn_test_hud", "anm_shoot", "shoot");

  let verdict: PlayerHudMotionVerdict = requests.verify(
    &hands(&[("actor_hud", &["idle", "shoot1", "shoot2"])]),
    XrayEngine::Vanilla,
  );

  assert!(verdict.messages.is_empty());
  assert!(verdict.invalid_hands.is_empty());
}

#[test]
fn hands_whose_model_is_missing_are_one_finding_and_judge_no_weapon() {
  // Anomaly's `[actor_hud_radiation]` names a hands model that is not shipped. That is its own problem, said once,
  // rather than one more message for every motion a weapon asks of it.
  let root: PathBuf = GeneratedResources::create_dir("player_hud/missing_model").unwrap();

  fs::create_dir_all(root.join("configs")).unwrap();
  fs::write(
    root.join("configs").join("system.ltx"),
    "[actor_hud_radiation]\nvisual = anomaly_weapons\\hands\\wpn_hand_radiation_suit\nposition = 0, 0, 0\n\
     orientation = 0, 0, 0\nancor_0 = a\nancor_1 = a\n\n[wpn_test]\nclass = WP_AK74\nweapon_class = assault_rifle\n\
     $scheme = $item_weapon\nhud = wpn_test_hud\n\n[wpn_test_hud]\nanm_idle = idle\n",
  )
  .unwrap();

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .unwrap();
  let result: GamedataPlayerHudAnimationsVerificationResult =
    PlayerHudAnimationsVerifier::new(&project, &GamedataProjectVerifyOptions::default())
      .verify()
      .unwrap();

  fs::remove_dir_all(root).unwrap();

  assert_eq!(result.get_summary(), "0/1 player HUD sections valid");
  assert_eq!(
    result
      .get_findings()
      .iter()
      .map(|finding| finding.message())
      .collect::<Vec<&str>>(),
    ["Player HUD section [actor_hud_radiation] visual 'anomaly_weapons\\hands\\wpn_hand_radiation_suit' is not found"]
  );
}
