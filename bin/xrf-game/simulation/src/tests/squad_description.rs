use xrf_game_config::{ConfigFile, ConfigIssues};

use crate::squad::squad_description::SquadDescription;
use crate::squad::squad_size::SquadSize;

/// The squad `[squad]` of `text` describes, and the issues its read found.
fn read(text: &str) -> (Option<SquadDescription>, ConfigIssues) {
  let file: ConfigFile = ConfigFile::parse("squads.ltx", text).expect("reads");
  let mut issues: ConfigIssues = ConfigIssues::default();
  let squad: Option<SquadDescription> = file
    .read_section("squad", &mut issues)
    .and_then(|mut section| SquadDescription::read(&mut section));

  (squad, issues)
}

#[test]
fn reads_a_simulation_squad_with_random_members() {
  let squad: SquadDescription =
    read("[squad]\nfaction = stalker\nnpc_random = sim_a, sim_b, sim_a\nnpc_in_squad = 2, 3\n")
      .0
      .expect("reads");

  assert_eq!(squad.section, "squad");
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
  .0
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
fn refuses_a_squad_with_no_faction_no_member_or_a_reversed_size_and_says_why() {
  for (text, issue) in [
    ("[squad]\nnpc = a\n", "squads.ltx [squad] faction: is missing"),
    (
      "[squad]\nfaction = stalker\n",
      "squads.ltx [squad]: names no member, neither npc nor npc_random",
    ),
    (
      "[squad]\nfaction = stalker\nnpc_random = a\nnpc_in_squad = 4, 2\n",
      "squads.ltx [squad] npc_in_squad: '4, 2' draws more at least than at most",
    ),
  ] {
    let (squad, issues) = read(text);

    assert!(squad.is_none(), "{text}");
    assert_eq!(issues.iter().map(ToString::to_string).collect::<Vec<_>>(), vec![issue]);
  }
}
