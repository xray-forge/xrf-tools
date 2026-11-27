use crate::config::simulation_config::SimulationConfig;
use crate::smart::smart_spawn::SmartSpawn;
use crate::tests::config_source::ConfigSource;

const SYSTEM: &str = concat!(
  "[stalker_novice]\nfaction = stalker\nnpc_random = sim_stalker, sim_stalker\nnpc_in_squad = 2, 3\n",
  "[bandit_novice]\nfaction = bandit\nnpc = sim_bandit, missing_bandit\n",
  "[sim_stalker]\nclass = AI_STL_S\n",
  "[sim_bandit]\nclass = AI_STL_S\n",
);

fn smart(name: &str, custom_data: &str) -> SmartSpawn {
  SmartSpawn {
    name: String::from(name),
    custom_data: String::from(custom_data),
  }
}

fn smarts() -> Vec<SmartSpawn> {
  vec![
    smart("zat_base", "[smart_terrain]\nsim_type = base\nmax_population = 3\n"),
    smart(
      "zat_sim_1",
      "[smart_terrain]\ncfg = scripts\\zaton\\smart\\zat_sim_1.ltx\n",
    ),
    smart("jup_sim_1", "[smart_terrain]\n"),
    smart("broken", "no section"),
  ]
}

fn source(simulation: &str) -> ConfigSource {
  ConfigSource::new(
    SYSTEM,
    &[
      ("misc\\simulation.ltx", simulation),
      (
        "misc\\squad_behaviours.ltx",
        "[default_behaviour]\nresource = 2\nlair = 0\n[bad]\nlair = many\n",
      ),
      (
        "misc\\simulation_objects_props.ltx",
        "[default]\nbase = 0\n[zat_base]:default\nbase = 2\nsim_avail = {+open} true, false\n",
      ),
      (
        "scripts\\zaton\\smart\\zat_sim_1.ltx",
        concat!(
          "[smart_terrain]\nmax_population = 1\nrespawn_params = list\n[list]\nbandits\n",
          "[bandits]\nspawn_squads = bandit_novice, absent_squad\nspawn_num = 1\n"
        ),
      ),
    ],
  )
}

fn levels() -> Vec<String> {
  vec![String::from("zaton"), String::from("labx8"), String::from("jupiter")]
}

#[test]
fn reads_start_squads_level_by_level_in_the_engines_line_order_skipping_a_level_without_any() {
  let config: SimulationConfig = SimulationConfig::read(
    &source(concat!(
      "[start_position_zaton]\nstalker_novice = zat_base, zat_sim_1\nbandit_novice = zat_sim_1\n",
      "[start_position_jupiter]\nstalker_novice = jup_sim_1, nowhere\n"
    )),
    &levels(),
    &smarts(),
  );
  let start: Vec<String> = config
    .get_start()
    .iter()
    .map(|start| format!("{}@{}", start.squad, start.smart))
    .collect();

  // labx8 has no section; vanilla stopped there and never reached jupiter.
  assert_eq!(
    start,
    vec![
      "bandit_novice@zat_sim_1",
      "stalker_novice@zat_base",
      "stalker_novice@zat_sim_1",
      "stalker_novice@jup_sim_1"
    ]
  );
  assert!(config.get_issues().iter().any(|issue| issue.contains("'nowhere'")));
}

#[test]
fn reads_anomalys_start_form_when_no_level_section_is_written() {
  let config: SimulationConfig = SimulationConfig::read(
    &source("[zat_base]\nstalker_novice = 2\nbandit_novice\n[unplaced]\nstalker_novice\n"),
    &levels(),
    &smarts(),
  );
  let start: Vec<String> = config
    .get_start()
    .iter()
    .map(|start| format!("{}@{}", start.squad, start.smart))
    .collect();

  assert_eq!(
    start,
    vec![
      "bandit_novice@zat_base",
      "stalker_novice@zat_base",
      "stalker_novice@zat_base"
    ]
  );
}

#[test]
fn reads_smarts_from_custom_data_or_their_cfg_and_keeps_what_does_not_read_as_issues() {
  let config: SimulationConfig = SimulationConfig::read(&source("[start_position_zaton]\n"), &levels(), &smarts());

  assert_eq!(
    config.get_smart("zat_base").map(|smart| smart.kind.as_str()),
    Some("base")
  );
  assert_eq!(
    config
      .get_smart("zat_sim_1")
      .map(|smart| smart.respawns[0].squads.clone()),
    Some(vec![String::from("bandit_novice"), String::from("absent_squad")])
  );
  assert!(config.get_smart("jup_sim_1").is_some());
  assert!(config.get_smart("broken").is_none());

  // Squads come from the respawns as well as the start; members and sections it lacks are issues.
  assert!(config.get_squad("bandit_novice").is_some());
  assert!(config.get_squad("absent_squad").is_none());

  let issues: String = config.get_issues().join("\n");

  assert!(issues.contains("'broken'"), "{issues}");
  assert!(issues.contains("[absent_squad]"), "{issues}");
  assert!(issues.contains("[missing_bandit]"), "{issues}");
  assert!(issues.contains("[bad]"), "{issues}");
}

#[test]
fn reads_behaviours_and_props_with_inheritance() {
  let config: SimulationConfig = SimulationConfig::read(&source("[start_position_zaton]\n"), &levels(), &smarts());

  assert_eq!(
    config
      .get_behaviour("default_behaviour")
      .and_then(|behaviour| behaviour.weights.get("resource").copied()),
    Some(2.0)
  );
  assert!(config.get_behaviour("bad").is_none());

  let props = config.get_props("zat_base").expect("the smart's props");

  assert_eq!(props.values.get("base"), Some(&2.0));
  assert_eq!(props.available.branches.len(), 2);
  assert_eq!(
    config
      .get_props("default")
      .map(|props| props.available.branches[0].result.clone()),
    Some(Some(String::from("true")))
  );
}
