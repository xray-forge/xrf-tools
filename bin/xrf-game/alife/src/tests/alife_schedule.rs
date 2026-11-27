use xrf_game_objects::{
  ObjectCreature, ObjectId, ObjectPlace, ObjectRegistry, ObjectSchedule, ObjectScheduleKind, ObjectSwitching,
};
use xrf_math::glam::Vec3;

use crate::schedule::alife_schedule::AlifeSchedule;
use crate::tests::fixtures::{OTHER_LEVEL, id, object, registry, schedule};

/// Ten objects, `scheduled` of them with schedule rows A-Life steers.
fn objects(scheduled: &[u32]) -> ObjectRegistry {
  let mut objects: ObjectRegistry = registry((0..10).map(|_| object(0.0, ObjectSwitching::ByDistance)).collect());

  for index in scheduled {
    schedule(&mut objects, *index);
  }

  objects
}

fn raw(ids: &[ObjectId]) -> Vec<u32> {
  ids.iter().map(|id| id.get_index()).collect()
}

#[test]
fn takes_a_fixed_number_in_id_order_and_takes_up_after_the_last_one_taken() {
  let objects: ObjectRegistry = objects(&[1, 3, 4, 6, 8]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  assert_eq!(raw(&schedule.take_due(&objects, 2)), vec![1, 3]);
  assert_eq!(schedule.get_next(), id(4));
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
  objects.bring_online(id(2), 0);
  objects.set_schedule(
    id(3),
    ObjectSchedule {
      kind: ObjectScheduleKind::Creature,
      direct_control: true,
      updated: None,
    },
  );
  objects.set_creature(id(3), ObjectCreature { health: 0.0 });
  objects.set_schedule(
    id(4),
    ObjectSchedule {
      kind: ObjectScheduleKind::SmartZone,
      direct_control: false,
      updated: None,
    },
  );

  assert_eq!(raw(&schedule.take_due(&objects, 10)), vec![1, 5]);

  // Back offline, it is taken again when the walk reaches it.
  objects.take_offline(id(2));

  assert_eq!(raw(&schedule.take_due(&objects, 10)), vec![1, 2, 5]);
}

#[test]
fn updates_offline_objects_of_every_level() {
  let mut objects: ObjectRegistry = objects(&[6]);

  objects.move_to(
    id(6),
    ObjectPlace {
      position: Vec3::ZERO,
      level: Some(OTHER_LEVEL),
      game_vertex: Some(0),
      level_vertex: Some(0),
    },
  );

  assert_eq!(raw(&AlifeSchedule::default().take_due(&objects, 10)), vec![6]);
}

#[test]
fn walks_past_a_released_object_to_its_slots_next_one() {
  let mut objects: ObjectRegistry = objects(&[1, 2, 3]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  assert_eq!(raw(&schedule.take_due(&objects, 2)), vec![1, 2]);

  // Object 2 is released and its slot taken by a newcomer a generation on, which the walk reaches in its turn.
  objects.release(id(2));

  let newcomer: ObjectId = objects
    .create(object(0.0, ObjectSwitching::ByDistance), None)
    .expect("a free slot");

  schedule.take_due(&objects, 1);
  objects.set_schedule(
    newcomer,
    ObjectSchedule {
      kind: ObjectScheduleKind::Group,
      direct_control: true,
      updated: None,
    },
  );

  assert_eq!(newcomer, ObjectId::new(2, 1).expect("a generation"));
  assert_eq!(schedule.take_due(&objects, 3), vec![id(1), newcomer, id(3)]);
}

#[test]
fn stamps_each_object_it_updates_with_the_tick() {
  let mut objects: ObjectRegistry = objects(&[1, 3, 5]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  assert_eq!(schedule.update(&mut objects, 2, 40).updated, 2);
  assert_eq!(schedule.update(&mut objects, 2, 67).updated, 2);

  let stamps: Vec<Option<u64>> = [1, 3, 5]
    .iter()
    .map(|index| objects.get_schedules().get(id(*index)).and_then(|row| row.updated))
    .collect();

  assert_eq!(stamps, vec![Some(67), Some(40), Some(67)]);
}

#[test]
fn takes_nothing_and_stays_put_when_nothing_needs_updating() {
  let objects: ObjectRegistry = objects(&[]);
  let mut schedule: AlifeSchedule = AlifeSchedule::default();

  assert!(schedule.take_due(&objects, 10).is_empty());
  assert_eq!(schedule.get_next(), id(0));
}
