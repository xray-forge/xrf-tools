use xrf_ltx::Ltx;

use crate::squad::squad_description::SquadDescription;
use crate::squad::squad_size::SquadSize;

fn read(text: &str) -> Result<SquadDescription, String> {
  let ltx: Ltx = Ltx::read_from_str(text).expect("reads");

  SquadDescription::read("squad", ltx.section("squad").expect("the section"))
}

#[test]
fn reads_a_simulation_squad_with_random_members() {
  let squad: SquadDescription =
    read("[squad]\nfaction = stalker\nnpc_random = sim_a, sim_b, sim_a\nnpc_in_squad = 2, 3\n").expect("reads");

  assert_eq!(squad.faction, "stalker");
  assert_eq!(squad.behaviour, "stalker");
  assert!(squad.members.is_empty());
  assert_eq!(squad.random_members, vec!["sim_a", "sim_b", "sim_a"]);
  assert_eq!(squad.size, SquadSize { min: 2, max: 3 });
  assert_eq!(squad.spawn_point.branches[0].result.as_deref(), Some("self"));
  assert_eq!(squad.target, None);
  assert!(!squad.always_walk && !squad.always_arrived);
}

#[test]
fn reads_a_story_squad_with_fixed_members_a_target_and_a_behaviour() {
  let squad: SquadDescription = read(concat!(
    "[squad]\nfaction = monster\nbehaviour = monster_predatory_day\nnpc = boss, guard\n",
    "target_smart = {+info} zat_a1, nil\nspawn_point = zat_sim_15_spawn\nalways_arrived = true\n"
  ))
  .expect("reads");

  assert_eq!(squad.behaviour, "monster_predatory_day");
  assert_eq!(squad.members, vec!["boss", "guard"]);
  assert_eq!(squad.size, SquadSize::default());
  assert_eq!(squad.target.map(|target| target.branches.len()), Some(2));
  assert_eq!(
    squad.spawn_point.branches[0].result.as_deref(),
    Some("zat_sim_15_spawn")
  );
  assert!(squad.always_arrived);
}

#[test]
fn refuses_a_squad_with_no_faction_no_member_or_a_reversed_size() {
  assert!(read("[squad]\nnpc = a\n").is_err());
  assert!(read("[squad]\nfaction = stalker\n").is_err());
  assert!(read("[squad]\nfaction = stalker\nnpc_random = a\nnpc_in_squad = 4, 2\n").is_err());
}
