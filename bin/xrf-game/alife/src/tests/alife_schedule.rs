use xrf_game_objects::{ObjectCreature, ObjectId, ObjectRegistry, ObjectSchedule, ObjectScheduleKind, ObjectSwitching};

use crate::schedule::alife_schedule::AlifeSchedule;
use crate::tests::fixtures::{OTHER_LEVEL, object, registry, schedule};

/// Ten objects, `scheduled` of them with schedule rows A-Life steers.
fn objects(scheduled: &[u16]) -> ObjectRegistry {
  let mut objects: ObjectRegistry = registry((0..10).map(|_| object(0.0, ObjectSwitching::ByDistance)).collect());

  for id in scheduled {
    schedule(&mut objects, *id);
  }

  objects
}

fn raw(ids: &[ObjectId]) -> Vec<u16> {
  ids.iter().map(|id| id.0).collect()
}

#[test]
fn takes_a_fixed_number_in_id_order_and_takes_up_after_the_last_one_taken() {
  let objects: ObjectRegistry = objects(&[1, 3, 4, 6, 8]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  assert_eq!(raw(&schedule.take_due(&objects, 2)), vec![1, 3]);
  assert_eq!(schedule.get_next(), ObjectId(4));
  assert_eq!(raw(&schedule.take_due(&objects, 2)), vec![4, 6]);
  // Past the last object the walk wraps round to the first.
  assert_eq!(raw(&schedule.take_due(&objects, 2)), vec![8, 1]);
  assert_eq!(raw(&schedule.take_due(&objects, 2)), vec![3, 4]);
}

#[test]
fn takes_no_object_twice_in_an_update_when_fewer_need_it_than_it_may_take() {
  let objects: ObjectRegistry = objects(&[2, 5, 7]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  schedule.take_due(&objects, 1);

  assert_eq!(raw(&schedule.take_due(&objects, 20)), vec![5, 7, 2]);
  assert_eq!(raw(&schedule.take_due(&objects, 20)), vec![5, 7, 2]);
}

#[test]
fn passes_over_objects_that_need_no_update_as_it_reaches_them() {
  let mut objects: ObjectRegistry = objects(&[1, 2, 3, 4, 5]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  // Online, a creature dead since its row was made, a smart zone a script steers.
  objects.bring_online(ObjectId(2), 0);
  objects.get_schedules_mut().insert(
    ObjectId(3),
    ObjectSchedule {
      kind: ObjectScheduleKind::Creature,
      direct_control: true,
      updated: None,
    },
  );
  objects
    .get_creatures_mut()
    .insert(ObjectId(3), ObjectCreature { health: 0.0 });
  objects.get_schedules_mut().insert(
    ObjectId(4),
    ObjectSchedule {
      kind: ObjectScheduleKind::SmartZone,
      direct_control: false,
      updated: None,
    },
  );

  assert_eq!(raw(&schedule.take_due(&objects, 10)), vec![1, 5]);

  // Back offline, it is taken again when the walk reaches it.
  objects.take_offline(ObjectId(2));

  assert_eq!(raw(&schedule.take_due(&objects, 10)), vec![1, 2, 5]);
}

#[test]
fn updates_offline_objects_of_every_level() {
  let mut objects: ObjectRegistry = objects(&[6]);

  if let Some(record) = objects.get_records_mut().get_mut(ObjectId(6)) {
    record.level = Some(OTHER_LEVEL);
  }

  assert_eq!(raw(&AlifeSchedule::default().take_due(&objects, 10)), vec![6]);
}

#[test]
fn stamps_each_object_it_updates_with_the_tick() {
  let mut objects: ObjectRegistry = objects(&[1, 3, 5]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  assert_eq!(schedule.update(&mut objects, 2, 40).updated, 2);
  assert_eq!(schedule.update(&mut objects, 2, 67).updated, 2);

  let stamps: Vec<Option<u64>> = [1, 3, 5]
    .iter()
    .map(|id| objects.get_schedules().get(ObjectId(*id)).and_then(|row| row.updated))
    .collect();

  assert_eq!(stamps, vec![Some(67), Some(40), Some(67)]);
}

#[test]
fn takes_nothing_and_stays_put_when_nothing_needs_updating() {
  let objects: ObjectRegistry = objects(&[]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  assert!(schedule.take_due(&objects, 10).is_empty());
  assert_eq!(schedule.get_next(), ObjectId(0));
}
