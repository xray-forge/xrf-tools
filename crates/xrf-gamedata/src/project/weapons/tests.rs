//! The weapons check over `system.ltx` trees written to disk, each problem a finding of its own.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::GeneratedResources;

use crate::project::gamedata_check::GamedataCheck;
use crate::project::weapons::verify_weapons_result::GamedataWeaponVerificationResult;
use crate::project::weapons::weapons_verifier::WeaponsVerifier;
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions};

/// A weapon section the check selects, with the fields given after its class keys.
const WEAPON: &str = "[wpn_test]\nclass = WP_AK74\nweapon_class = assault_rifle\n$scheme = $item_weapon\n";

fn verify(name: &str, system_ltx: &str, sounds: &[&str]) -> GamedataWeaponVerificationResult {
  let root: PathBuf = GeneratedResources::create_dir(&format!("weapons/{name}")).unwrap();

  fs::create_dir_all(root.join("configs")).unwrap();
  fs::write(root.join("configs").join("system.ltx"), system_ltx).unwrap();

  for sound in sounds {
    let path: PathBuf = root.join("sounds").join(format!("{sound}.ogg"));

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, [0]).unwrap();
  }

  let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
    root: root.clone(),
    output: xrf_output::OutputOptions::default(),
    ..Default::default()
  })
  .unwrap();
  let result: GamedataWeaponVerificationResult =
    WeaponsVerifier::new(&project, &GamedataProjectVerifyOptions::default())
      .verify()
      .unwrap();

  fs::remove_dir_all(root).unwrap();

  result
}

fn messages(result: &GamedataWeaponVerificationResult) -> Vec<&str> {
  result.get_findings().iter().map(|finding| finding.message()).collect()
}

#[test]
fn reports_each_problem_of_a_section_as_its_own_finding() {
  let result: GamedataWeaponVerificationResult = verify(
    "bare",
    &format!("{WEAPON}visual = weapons\\missing\nhud = wpn_test_hud\nsnd_shoot = weapons\\shot\n"),
    &[],
  );

  assert_eq!(result.get_summary(), "0/1 weapons valid");
  assert_eq!(
    messages(&result),
    [
      "Weapon section [wpn_test] has no snd_draw, a sound every weapon plays",
      "Weapon section [wpn_test] has no snd_empty, a sound every weapon plays",
      "Weapon section [wpn_test] has no snd_holster, a sound every weapon plays",
      "Weapon section [wpn_test] has no snd_reload, a sound every weapon plays",
      "Weapon section [wpn_test] hud section 'wpn_test_hud' is not defined",
      "Weapon section [wpn_test] snd_shoot names missing sound 'weapons\\shot'",
      "Weapon section [wpn_test] visual 'weapons\\missing' is not found",
    ]
  );
}

#[test]
fn reports_a_broken_layered_sound_section_and_skips_a_disabled_sound() {
  let result: GamedataWeaponVerificationResult = verify(
    "layers",
    &format!(
      "{WEAPON}snd_draw = $no_sound\nsnd_empty = $no_sound\nsnd_holster = $no_sound\nsnd_reload = $no_sound\n\
       snd_shoot = shot_layers, 1.0\n\
       [shot_layers]\nsnd_1_layer = weapons\\shot\nsnd_3_layer = weapons\\missing\n"
    ),
    &["weapons\\shot"],
  );

  let messages: Vec<&str> = messages(&result);

  assert!(messages.contains(
    &"Weapon section [wpn_test] snd_shoot gives layered sound section [shot_layers] a volume or delay, which a layered sound cannot take"
  ));
  assert!(messages.contains(
    &"Weapon section [wpn_test] snd_shoot: Sound layer section [shot_layers] expects snd_2_layer before snd_3_layer"
  ));
  assert!(messages.contains(
    &"Weapon section [wpn_test] snd_shoot: sound layer section [shot_layers] snd_3_layer names missing sound 'weapons\\missing'"
  ));
  assert!(!messages.iter().any(|message| message.contains("$no_sound")));
}
