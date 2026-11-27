use xrf_math::glam::Vec3;
use xrf_spawn::{ClsId, SpawnPlacedObjects};

use crate::id::object_id::ObjectId;
use crate::record::object_ammo::ObjectAmmo;
use crate::record::object_creature::ObjectCreature;
use crate::record::object_record::ObjectRecord;
use crate::record::object_schedule::ObjectSchedule;
use crate::record::object_schedule_kind::ObjectScheduleKind;
use crate::record::object_switching::ObjectSwitching;
use crate::registry::object_registry::ObjectRegistry;

pub(crate) fn record(level: u8, parent: Option<ObjectId>) -> ObjectRecord {
  ObjectRecord {
    section: String::from("stalker"),
    name: String::from("stalker_0001"),
    class: ClsId::AiStlS,
    position: Vec3::ZERO,
    direction: Vec3::Z,
    level: Some(level),
    game_vertex: Some(1),
    level_vertex: Some(2),
    parent,
    story_id: None,
    spawn_id: 0,
    switching: ObjectSwitching::ByDistance,
  }
}

/// Puts an object at `id` with a schedule row of `kind`, and a creature row of `health` if given.
fn scheduled(
  registry: &mut ObjectRegistry,
  id: u16,
  kind: ObjectScheduleKind,
  direct_control: bool,
  health: Option<f32>,
) {
  registry.insert(ObjectId(id), record(108, None));
  registry.get_schedules_mut().insert(
    ObjectId(id),
    ObjectSchedule {
      kind,
      direct_control,
      updated: None,
    },
  );

  if let Some(health) = health {
    registry
      .get_creatures_mut()
      .insert(ObjectId(id), ObjectCreature { health });
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
fn counts_the_objects_standing_on_a_level() {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  registry.insert(ObjectId(0), record(108, None));
  registry.insert(ObjectId(5), record(108, None));
  registry.insert(ObjectId(9), record(118, None));

  assert_eq!(registry.count_on_level(108), 2);
  assert_eq!(registry.count_on_level(118), 1);
  assert_eq!(registry.count_on_level(7), 0);
}

#[test]
fn brings_an_object_online_and_offline_with_what_it_holds() {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  registry.insert(ObjectId(1), record(108, None));
  registry.insert(ObjectId(2), record(108, Some(ObjectId(1))));
  registry.insert(ObjectId(3), record(108, Some(ObjectId(2))));
  registry.insert(ObjectId(4), record(108, None));

  assert_eq!(registry.get_children(ObjectId(1)), [ObjectId(2)]);

  registry.bring_online(ObjectId(1), 7);

  assert!(registry.is_online(ObjectId(1)) && registry.is_online(ObjectId(2)) && registry.is_online(ObjectId(3)));
  assert!(!registry.is_online(ObjectId(4)));
  assert_eq!(registry.get_online().get(ObjectId(3)).map(|row| row.since), Some(7));

  // Already online, it keeps the tick it came at.
  registry.bring_online(ObjectId(1), 9);
  assert_eq!(registry.get_online().get(ObjectId(1)).map(|row| row.since), Some(7));

  registry.take_offline(ObjectId(1));

  assert!(registry.get_online().is_empty());
}

#[test]
fn lets_neither_a_dead_creature_nor_an_empty_ammo_box_go_offline() {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  for id in 0..5 {
    registry.insert(ObjectId(id), record(108, None));
  }

  registry
    .get_creatures_mut()
    .insert(ObjectId(1), ObjectCreature { health: 0.7 });
  registry
    .get_creatures_mut()
    .insert(ObjectId(2), ObjectCreature { health: 0.0 });
  registry.get_ammo_mut().insert(ObjectId(3), ObjectAmmo { rounds: 30 });
  registry.get_ammo_mut().insert(ObjectId(4), ObjectAmmo { rounds: 0 });

  // A thing in no concern table, a living creature and a full box may; a dead creature and an empty box may not.
  assert!(registry.may_go_offline(ObjectId(0)));
  assert!(registry.may_go_offline(ObjectId(1)));
  assert!(!registry.may_go_offline(ObjectId(2)));
  assert!(registry.may_go_offline(ObjectId(3)));
  assert!(!registry.may_go_offline(ObjectId(4)));
}

#[test]
fn needs_updating_offline_by_kind_control_and_health() {
  let mut registry: ObjectRegistry = ObjectRegistry::default();

  scheduled(&mut registry, 0, ObjectScheduleKind::Creature, true, Some(0.7));
  scheduled(&mut registry, 1, ObjectScheduleKind::Creature, true, Some(0.000_05));
  scheduled(&mut registry, 2, ObjectScheduleKind::Creature, false, Some(0.7));
  scheduled(&mut registry, 3, ObjectScheduleKind::SmartZone, true, None);
  scheduled(&mut registry, 4, ObjectScheduleKind::SmartZone, false, None);
  scheduled(&mut registry, 5, ObjectScheduleKind::Group, false, None);
  scheduled(&mut registry, 6, ObjectScheduleKind::Creature, true, Some(0.7));
  registry.insert(ObjectId(7), record(108, None));
  registry.bring_online(ObjectId(6), 0);

  let needs: Vec<bool> = (0..8).map(|id| registry.needs_update(ObjectId(id))).collect();

  // A living creature A-Life steers, a smart zone it steers and any group do; one barely alive, one a script steers,
  // an online one and one with no schedule row do not.
  assert_eq!(needs, vec![true, false, false, true, false, true, false, false]);
}
