use xrf_game_graph::{GameGraph, GamePatrols};
use xrf_game_hooks::HookRegistry;
use xrf_game_objects::{ObjectId, ObjectRecord, ObjectRegistry, ObjectScheduleKind, ObjectSwitching};
use xrf_math::glam::Vec3;
use xrf_spawn::ClsId;

use crate::config::simulation_config::SimulationConfig;
use crate::smart::smart_spawn::SmartSpawn;
use crate::start::simulation_start::SimulationStart;
use crate::start::simulation_start_report::SimulationStartReport;
use crate::tests::config_source::TextSource;

const SYSTEM: &str = concat!(
  "[stalker_novice]\nclass = ON_OFF_S\nfaction = stalker\nnpc_random = sim_a, sim_b, sim_b\nnpc_in_squad = 2, 3\n",
  "[story_squad]\nclass = ON_OFF_S\nfaction = bandit\nnpc = boss, sim_a\nspawn_point = absent_patrol\n",
  "[sim_a]\nclass = AI_STL_S\n[sim_b]\nclass = AI_STL_S\n[boss]\nclass = AI_STL_S\n",
);

/// A smart terrain record named `name` at `x` metres along `+x`, on game vertex 3.
fn smart(name: &str, x: f32) -> ObjectRecord {
  ObjectRecord {
    section: String::from("smart_terrain"),
    name: String::from(name),
    class: ClsId::SmrtTrrn,
    position: Vec3::new(x, 0.0, 0.0),
    direction: Vec3::Z,
    game_vertex: Some(3),
    level_vertex: Some(9),
    story_id: None,
    spawn_id: 0,
    switching: ObjectSwitching::AlwaysOnline,
  }
}

fn config(start: &str) -> SimulationConfig {
  let smarts: Vec<SmartSpawn> = ["zat_base", "zat_sim_1"]
    .iter()
    .map(|name| SmartSpawn {
      name: String::from(*name),
      custom_data: String::from("[smart_terrain]\n"),
    })
    .collect();

  SimulationConfig::read(
    &TextSource::new(
      SYSTEM,
      &[
        ("misc\\simulation.ltx", start),
        ("misc\\squad_behaviours.ltx", ""),
        ("misc\\simulation_objects_props.ltx", ""),
      ],
    ),
    &[String::from("zaton")],
    &smarts,
  )
}

/// A start's objects and what it reported.
struct Started {
  objects: ObjectRegistry,
  report: SimulationStartReport,
}

/// Two smart terrains at ids 0 and 1, and the start `config` names run on them under `seed`.
fn start(config: &SimulationConfig, seed: u64) -> Started {
  let mut objects: ObjectRegistry = ObjectRegistry::default();

  objects.create(smart("zat_base", 10.0), Some(1));
  objects.create(smart("zat_sim_1", 50.0), Some(1));

  let report: SimulationStartReport = SimulationStart {
    config,
    graph: &GameGraph::default(),
    patrols: &GamePatrols::default(),
    hooks: &HookRegistry::default(),
    seed,
  }
  .create(&mut objects);

  Started { objects, report }
}

#[test]
fn creates_each_start_squad_then_its_members_at_its_smart_and_assigns_it() {
  let config: SimulationConfig = config("[start_position_zaton]\nstalker_novice = zat_sim_1\n");
  let Started { objects, report } = start(&config, 7);
  let members: Vec<ObjectId> = objects.get_group_members(ObjectId::of_index(2)).to_vec();

  assert_eq!(report.squads, 1);
  assert!((2..=3).contains(&members.len()), "{members:?}");
  assert_eq!(report.members, members.len());
  // The squad takes the next id, its members the ones after.
  assert_eq!(members[0], ObjectId::of_index(3));

  let squad: &ObjectRecord = objects.get_records().get(ObjectId::of_index(2)).expect("the squad");

  assert_eq!(squad.name, "stalker_novice0002");
  assert_eq!(squad.class, ClsId::OnOffS);
  assert_eq!(squad.switching, ObjectSwitching::Group);
  assert_eq!(squad.position, Vec3::new(50.0, 0.0, 0.0));
  assert_eq!(squad.game_vertex, Some(3));
  assert_eq!(
    objects
      .get_groups()
      .get(ObjectId::of_index(2))
      .and_then(|group| group.smart),
    Some(ObjectId::of_index(1))
  );
  assert_eq!(
    objects.get_schedules().get(ObjectId::of_index(2)).map(|row| row.kind),
    Some(ObjectScheduleKind::Group)
  );

  for member in members {
    let record: &ObjectRecord = objects.get_records().get(member).expect("the member");

    assert!(record.section == "sim_a" || record.section == "sim_b");
    assert_eq!(record.switching, ObjectSwitching::ByDistance);
    assert_eq!(record.position, squad.position);
    assert_eq!(objects.get_creatures().get(member).map(|row| row.health), Some(1.0));
    assert_eq!(
      objects.get_members().get(member).map(|row| row.group),
      Some(ObjectId::of_index(2))
    );
    assert!(!objects.needs_update(member));
  }
}

#[test]
fn draws_the_same_members_for_the_same_seed_and_others_for_another() {
  let config: SimulationConfig = config("[start_position_zaton]\nstalker_novice = zat_base, zat_sim_1\n");
  let sections = |objects: &ObjectRegistry| -> Vec<String> {
    objects
      .get_records()
      .iter()
      .map(|(_, record)| record.section.clone())
      .collect()
  };
  let first: Vec<String> = sections(&start(&config, 11).objects);

  assert_eq!(first, sections(&start(&config, 11).objects));
  assert!((0..20).any(|seed| sections(&start(&config, seed).objects) != first));
}

#[test]
fn creates_fixed_members_once_and_puts_a_squad_spawning_at_a_missing_patrol_at_its_smart() {
  let config: SimulationConfig = config("[start_position_zaton]\nstory_squad = zat_base\n");
  let Started { objects, report } = start(&config, 1);
  let members: Vec<String> = objects
    .get_group_members(ObjectId::of_index(2))
    .iter()
    .filter_map(|id| objects.get_records().get(*id).map(|record| record.section.clone()))
    .collect();

  assert_eq!(members, vec!["boss", "sim_a"]);
  assert_eq!(
    objects
      .get_records()
      .get(ObjectId::of_index(3))
      .map(|record| record.position),
    Some(Vec3::new(10.0, 0.0, 0.0))
  );
  assert!(
    report.issues.iter().any(|issue| issue.contains("absent_patrol")),
    "{:?}",
    report.issues
  );
}

#[test]
fn reports_a_start_squad_whose_smart_the_spawn_lacks() {
  let config: SimulationConfig = config("[start_position_zaton]\nstalker_novice = zat_base\n");
  let mut objects: ObjectRegistry = ObjectRegistry::default();
  let report: SimulationStartReport = SimulationStart {
    config: &config,
    graph: &GameGraph::default(),
    patrols: &GamePatrols::default(),
    hooks: &HookRegistry::default(),
    seed: 0,
  }
  .create(&mut objects);

  assert_eq!(report.squads, 0);
  assert!(objects.get_records().is_empty());
  assert!(report.issues[0].contains("'zat_base'"), "{:?}", report.issues);
}
