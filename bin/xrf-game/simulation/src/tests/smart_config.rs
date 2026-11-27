use xrf_ltx::Ltx;

use crate::smart::smart_config::SmartConfig;

fn read(text: &str) -> Result<SmartConfig, String> {
  SmartConfig::read("zat_sim_13", &Ltx::read_from_str(text).expect("reads"))
}

#[test]
fn reads_a_smart_with_its_respawn_sections_in_the_engines_order() {
  let smart: SmartConfig = read(concat!(
    "[smart_terrain]\nsquad_id = 23\nmax_population = 1\nrespawn_params = respawn@zat_sim_13\n",
    "[respawn@zat_sim_13]\nspawn_bandit@novice\nspawn_bandit@advanced\n",
    "[spawn_bandit@novice]\nspawn_squads = bandit_novice\nspawn_num = {-searched} 2, 0\n",
    "[spawn_bandit@advanced]\nspawn_squads = bandit_novice, bandit_advanced\nspawn_num = 1\n"
  ))
  .expect("reads");

  assert_eq!(smart.kind, "default");
  assert_eq!(smart.squad_id, 23);
  assert_eq!(smart.max_population.branches[0].result.as_deref(), Some("1"));
  assert_eq!(
    smart
      .respawns
      .iter()
      .map(|respawn| respawn.section.as_str())
      .collect::<Vec<_>>(),
    vec!["spawn_bandit@advanced", "spawn_bandit@novice"]
  );
  assert_eq!(smart.respawns[0].squads, vec!["bandit_novice", "bandit_advanced"]);
  assert_eq!(smart.respawns[1].count.branches.len(), 2);
  assert_eq!(smart.respawn_radius, 150.0);
  assert_eq!(smart.respawn_idle, 1_000.0);
  assert_eq!(smart.arrive_distance, 30.0);
}

#[test]
fn keeps_vanillas_defaults_and_reads_what_a_tree_sets() {
  let smart: SmartConfig = read(concat!(
    "[smart_terrain]\nsim_type = resource\nrespawn_radius = 300\nrespawn_idle = 43200\narrive_dist = 50\n",
    "respawn_only_smart = true\nmutant_lair = true\n"
  ))
  .expect("reads");

  assert_eq!(smart.kind, "resource");
  assert_eq!(smart.max_population.branches[0].result.as_deref(), Some("0"));
  assert!(smart.respawns.is_empty());
  assert_eq!(smart.respawn_radius, 300.0);
  assert_eq!(smart.respawn_idle, 43_200.0);
  assert_eq!(smart.arrive_distance, 50.0);
  assert!(smart.respawn_only && smart.mutant_lair && !smart.no_mutant);
}

#[test]
fn refuses_a_smart_without_its_section_a_wrong_kind_or_a_missing_respawn_section() {
  assert!(read("[other]\n").is_err());
  assert!(read("[smart_terrain]\nsim_type = lair\n").is_err());
  assert!(read("[smart_terrain]\nrespawn_params = absent\n").is_err());
  assert!(read("[smart_terrain]\nrespawn_params = list\n[list]\nmissing\n").is_err());
  assert!(read("[smart_terrain]\nrespawn_params = list\n[list]\nkind\n[kind]\nspawn_squads = a\n").is_err());
}
