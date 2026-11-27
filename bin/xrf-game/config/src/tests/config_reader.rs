use std::collections::BTreeMap;

use crate::file::config_file::ConfigFile;
use crate::issue::config_issues::ConfigIssues;
use crate::read::config_reader::ConfigReader;

fn file(text: &str) -> ConfigFile {
  ConfigFile::parse("test.ltx", text).expect("the test config reads")
}

fn lines(issues: &ConfigIssues) -> Vec<String> {
  issues.iter().map(ToString::to_string).collect()
}

#[test]
fn takes_an_optional_keys_default_silently_and_says_a_required_one_is_missing() {
  let file: ConfigFile = file("[actor]\nwalk_accel = 12\n");
  let mut issues: ConfigIssues = ConfigIssues::default();
  let mut actor: ConfigReader<'_, '_> = file.read_section("actor", &mut issues).expect("the section");

  assert_eq!(actor.require_or("walk_accel", 1.0), 12.0);
  assert_eq!(actor.read_or("walk_strafe_coef", 1.0), 1.0);
  assert_eq!(actor.require_or("jump_speed", 6.0), 6.0);
  assert_eq!(lines(&issues), vec!["test.ltx [actor] jump_speed: is missing"]);
}

#[test]
fn says_a_key_not_of_its_kind_and_takes_the_default() {
  let file: ConfigFile = file("[alife]\nswitch_distance = far\nobjects_per_update = 20\n");
  let mut issues: ConfigIssues = ConfigIssues::default();
  let mut alife: ConfigReader<'_, '_> = file.read_section("alife", &mut issues).expect("the section");

  assert_eq!(alife.read_or("switch_distance", 150.0_f32), 150.0);
  assert_eq!(alife.require_or("objects_per_update", 10_u32), 20);
  assert_eq!(
    lines(&issues),
    vec!["test.ltx [alife] switch_distance: 'far' is no number"]
  );
}

#[test]
fn says_a_required_section_is_missing_and_an_optional_one_not() {
  let file: ConfigFile = file("[alife]\n");
  let mut issues: ConfigIssues = ConfigIssues::default();

  assert!(file.read_section("hud_sound", &mut issues).is_none());
  assert!(file.require_section("actor", &mut issues).is_none());
  assert_eq!(lines(&issues), vec!["test.ltx [actor]: is missing"]);
}

#[test]
fn walks_keys_in_the_engines_order_and_reads_names_and_every_key() {
  let file: ConfigFile = file("[weights]\nzeta = 2\nalpha = 1\nbeta = x\n[squad]\nnpc = sim_a, sim_b|sim_c\n");
  let mut issues: ConfigIssues = ConfigIssues::default();
  let mut weights: ConfigReader<'_, '_> = file.read_section("weights", &mut issues).expect("the section");

  assert_eq!(weights.get_keys(), vec!["alpha", "beta", "zeta"]);
  assert_eq!(
    weights.read_all::<f32>(&[]),
    BTreeMap::from([(String::from("alpha"), 1.0), (String::from("zeta"), 2.0)])
  );
  assert_eq!(issues.len(), 1);

  let squad: ConfigReader<'_, '_> = file.read_section("squad", &mut issues).expect("the section");

  assert_eq!(squad.read_names("npc"), vec!["sim_a", "sim_b", "sim_c"]);
  assert!(squad.read_names("npc_random").is_empty());
}
