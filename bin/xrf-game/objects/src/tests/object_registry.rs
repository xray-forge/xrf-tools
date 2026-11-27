use xrf_math::glam::Vec3;
use xrf_spawn::{ClsId, SpawnPlacedObjects};

use crate::id::object_id::ObjectId;
use crate::record::object_ammo::ObjectAmmo;
use crate::record::object_creature::ObjectCreature;
use crate::record::object_place::ObjectPlace;
use crate::record::object_record::ObjectRecord;
use crate::record::object_schedule::ObjectSchedule;
use crate::record::object_schedule_kind::ObjectScheduleKind;
use crate::record::object_switching::ObjectSwitching;
use crate::registry::object_registry::ObjectRegistry;

const LEVEL: Option<u8> = Some(108);

fn id(index: u32) -> ObjectId {
  ObjectId::of_index(index)
}

pub(crate) fn record() -> ObjectRecord {
  ObjectRecord {
    section: String::from("stalker"),
    name: String::from("stalker_0001"),
    class: ClsId::AiStlS,
    position: Vec3::ZERO,
    direction: Vec3::Z,
    game_vertex: Some(1),
    level_vertex: Some(2),
    story_id: None,
    spawn_id: 0,
    switching: ObjectSwitching::ByDistance,
  }
}

/// A registry of `count` objects on the level, at ids 0 on.
fn registry_of(count: u32) -> ObjectRegistry {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  for index in 0..count {
    assert!(registry.insert(id(index), record(), LEVEL));
  }

  registry
}

/// Gives object `index` a schedule row of `kind`, and a creature row of `health` if given.
fn schedule(
  registry: &mut ObjectRegistry,
  index: u32,
  kind: ObjectScheduleKind,
  direct_control: bool,
  health: Option<f32>,
) {
  registry.set_schedule(
    id(index),
    ObjectSchedule {
      kind,
      direct_control,
      updated: None,
    },
  );

  if let Some(health) = health {
    registry.set_creature(id(index), ObjectCreature { health });
  }
}

#[test]
fn builds_an_empty_registry_from_a_spawn_with_no_objects() {
  let registry: ObjectRegistry = ObjectRegistry::of_spawn(&SpawnPlacedObjects {
    levels: Vec::new(),
    objects: Vec::new(),
    total: 0,
    skipped: Vec::new(),
  });

  assert!(registry.get_records().is_empty());
}

#[test]
fn indexes_the_objects_standing_on_each_level_as_they_move() {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  registry.insert(id(0), record(), LEVEL);
  registry.insert(id(5), record(), LEVEL);
  registry.insert(id(9), record(), Some(118));

  assert_eq!(registry.count_on_level(108), 2);
  assert_eq!(registry.count_on_level(118), 1);
  assert_eq!(registry.count_on_level(7), 0);
  assert_eq!(registry.get_level_objects(108).collect::<Vec<_>>(), vec![id(0), id(5)]);

  assert!(registry.move_to(
    id(0),
    ObjectPlace {
      position: Vec3::X,
      level: Some(118),
      game_vertex: Some(40),
      level_vertex: Some(41),
    }
  ));

  assert_eq!(registry.get_level_objects(118).collect::<Vec<_>>(), vec![id(0), id(9)]);
  assert_eq!(registry.get_level(id(0)), Some(118));
  assert_eq!(
    registry.get_records().get(id(0)).map(|record| record.game_vertex),
    Some(Some(40))
  );
  assert!(!registry.move_to(
    id(3),
    ObjectPlace {
      position: Vec3::ZERO,
      level: None,
      game_vertex: None,
      level_vertex: None,
    }
  ));
}

#[test]
fn puts_nothing_at_an_id_whose_slot_an_object_holds() {
  let mut registry: ObjectRegistry = registry_of(1);

  assert!(!registry.insert(id(0), record(), Some(118)));
  assert!(!registry.insert(ObjectId::new(0, 1).expect("a generation"), record(), Some(118)));
  assert_eq!(registry.get_level(id(0)), LEVEL);
}

#[test]
fn brings_an_object_online_and_offline_with_what_it_holds() {
  let mut registry: ObjectRegistry = registry_of(5);

  registry.set_parent(id(2), Some(id(1)));
  registry.set_parent(id(3), Some(id(2)));

  assert_eq!(registry.get_children(id(1)), [id(2)]);
  assert_eq!(registry.bring_online(id(1), 7), vec![id(1), id(2), id(3)]);
  assert!(!registry.is_online(id(4)));
  assert_eq!(registry.get_online().get(id(3)).map(|row| row.since), Some(7));

  // Already online, it keeps the tick it came at, and comes again as none.
  assert_eq!(registry.bring_online(id(1), 9), Vec::<ObjectId>::new());
  assert_eq!(registry.get_online().get(id(1)).map(|row| row.since), Some(7));

  assert_eq!(registry.take_offline(id(1)).len(), 3);
  assert!(registry.get_online().is_empty());
}

#[test]
fn moves_what_an_object_holds_and_refuses_a_loop() {
  let mut registry: ObjectRegistry = registry_of(3);

  assert!(registry.set_parent(id(1), Some(id(0))));
  assert!(registry.set_parent(id(2), Some(id(1))));
  // What an object holds cannot come to hold it.
  assert!(!registry.set_parent(id(0), Some(id(2))));
  assert!(!registry.set_parent(id(0), Some(id(0))));
  assert!(!registry.set_parent(id(0), Some(id(9))));

  assert!(registry.set_parent(id(1), None));
  assert_eq!(registry.get_parent(id(1)), None);
  assert!(registry.get_children(id(0)).is_empty());
  assert!(registry.set_parent(id(1), Some(id(0))));
  assert_eq!(registry.get_children(id(0)), [id(1)]);
}

#[test]
fn lets_neither_a_dead_creature_nor_an_empty_ammo_box_go_offline() {
  let mut registry: ObjectRegistry = registry_of(5);

  registry.set_creature(id(1), ObjectCreature { health: 0.7 });
  registry.set_creature(id(2), ObjectCreature { health: 0.0 });
  registry.set_ammo(id(3), ObjectAmmo { rounds: 30 });
  registry.set_ammo(id(4), ObjectAmmo { rounds: 0 });

  // A thing in no concern table, a living creature and a full box may; a dead creature and an empty box may not.
  assert!(registry.may_go_offline(id(0)));
  assert!(registry.may_go_offline(id(1)));
  assert!(!registry.may_go_offline(id(2)));
  assert!(registry.may_go_offline(id(3)));
  assert!(!registry.may_go_offline(id(4)));
  // No row for an object that does not exist.
  assert!(!registry.set_creature(id(9), ObjectCreature { health: 1.0 }));
}

#[test]
fn creates_objects_past_every_id_handed_out() {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  assert_eq!(registry.create(record(), LEVEL), Some(id(0)));
  assert_eq!(registry.create(record(), LEVEL), Some(id(1)));
  assert_eq!(registry.get_records().len(), 2);
  assert!(!registry.is_online(id(1)));

  // An object put at an id moves creation past it.
  registry.insert(id(40), record(), LEVEL);
  assert_eq!(registry.create(record(), LEVEL), Some(id(41)));
}

#[test]
fn releases_an_object_with_what_it_holds_and_hands_its_slot_out_a_generation_on() {
  let mut registry: ObjectRegistry = registry_of(4);

  registry.set_parent(id(1), Some(id(0)));
  registry.set_creature(id(1), ObjectCreature { health: 1.0 });
  registry.join_group(id(3), id(0));
  registry.bring_online(id(3), 0);

  assert!(registry.release(id(0)));
  assert!(!registry.release(id(0)));

  // Gone from every table and index, and from its squad.
  assert_eq!(registry.get_records().len(), 2);
  assert!(registry.get_creatures().is_empty());
  assert!(registry.get_group_members(id(3)).is_empty());
  assert!(registry.get_members().is_empty());
  assert_eq!(registry.get_online().len(), 1);
  assert_eq!(registry.get_level_objects(108).collect::<Vec<_>>(), vec![id(2), id(3)]);

  let reused: Option<ObjectId> = registry.create(record(), None);

  // The lowest freed slot, a generation on: the old id names nothing of the newcomer.
  assert_eq!(reused, ObjectId::new(0, 1));
  assert!(registry.get_records().get(id(0)).is_none());
  assert_eq!(registry.create(record(), None), ObjectId::new(1, 1));
  assert_eq!(registry.create(record(), None), Some(id(4)));
}

#[test]
fn releases_a_squad_and_lets_its_members_go() {
  let mut registry: ObjectRegistry = registry_of(3);

  registry.form_group(id(0), Some(id(2)));
  registry.join_group(id(0), id(1));

  assert!(registry.release(id(0)));
  assert!(registry.get_members().is_empty());
  assert!(registry.get_records().contains(id(1)));
}

#[test]
fn switches_a_squad_with_its_members_and_keeps_members_off_the_schedule() {
  let mut registry: ObjectRegistry = registry_of(4);

  schedule(&mut registry, 0, ObjectScheduleKind::Group, true, None);
  schedule(&mut registry, 1, ObjectScheduleKind::Creature, true, Some(1.0));
  schedule(&mut registry, 2, ObjectScheduleKind::Creature, true, Some(1.0));
  registry.set_parent(id(3), Some(id(2)));
  registry.form_group(id(0), Some(id(3)));
  registry.join_group(id(0), id(1));
  registry.join_group(id(0), id(2));

  assert_eq!(registry.get_group_members(id(0)), [id(1), id(2)]);
  assert_eq!(
    registry.get_groups().get(id(0)).and_then(|group| group.smart),
    Some(id(3))
  );
  assert_eq!(
    registry.get_members().get(id(2)).map(|member| member.group),
    Some(id(0))
  );
  // The squad is updated for its members.
  assert!(registry.needs_update(id(0)));
  assert!(!registry.needs_update(id(1)));

  registry.bring_online(id(0), 5);

  // Members and what they hold go with the squad.
  assert_eq!(registry.get_online().len(), 4);

  registry.take_offline(id(0));

  assert!(registry.get_online().is_empty());
}

#[test]
fn moves_a_member_from_one_squad_to_another() {
  let mut registry: ObjectRegistry = registry_of(3);

  registry.join_group(id(0), id(2));
  registry.join_group(id(1), id(2));

  assert!(registry.get_group_members(id(0)).is_empty());
  assert_eq!(registry.get_group_members(id(1)), [id(2)]);
  assert!(!registry.join_group(id(1), id(1)));
}

#[test]
fn needs_updating_offline_by_kind_control_and_health() {
  let mut registry: ObjectRegistry = registry_of(8);

  schedule(&mut registry, 0, ObjectScheduleKind::Creature, true, Some(0.7));
  schedule(&mut registry, 1, ObjectScheduleKind::Creature, true, Some(0.000_05));
  schedule(&mut registry, 2, ObjectScheduleKind::Creature, false, Some(0.7));
  schedule(&mut registry, 3, ObjectScheduleKind::SmartZone, true, None);
  schedule(&mut registry, 4, ObjectScheduleKind::SmartZone, false, None);
  schedule(&mut registry, 5, ObjectScheduleKind::Group, false, None);
  schedule(&mut registry, 6, ObjectScheduleKind::Creature, true, Some(0.7));
  registry.bring_online(id(6), 0);

  let needs: Vec<bool> = (0..8).map(|index| registry.needs_update(id(index))).collect();

  // A living creature A-Life steers, a smart zone it steers and any group do; one barely alive, one a script steers,
  // an online one and one with no schedule row do not.
  assert_eq!(needs, vec![true, false, false, true, false, true, false, false]);
}
