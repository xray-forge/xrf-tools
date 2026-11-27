use std::time::Duration;

use xrf_game_loop::{GameClock, LoopSystem, LoopTime};
use xrf_game_objects::{ObjectId, ObjectRegistry, ObjectSwitching};
use xrf_math::glam::Vec3;

use crate::alife::Alife;
use crate::alife_context::AlifeContext;
use crate::settings::alife_settings::AlifeSettings;
use crate::tests::fixtures::{LEVEL, object, registry, schedule};

/// Steps A-Life `ticks` ticks of 10 ms from tick `from`, the actor at `actor_x`, answering the ticks a sweep ran at.
fn run(alife: &mut Alife, objects: &mut ObjectRegistry, from: u64, ticks: u64, actor_x: f32) -> Vec<u64> {
  let mut swept: Vec<u64> = Vec::new();

  for tick in from..from + ticks {
    let time: LoopTime = LoopTime {
      tick,
      ..LoopTime::new(Duration::from_millis(10), GameClock::default())
    };
    let before = *alife.get_last_switch();

    alife.step(
      &time,
      AlifeContext {
        objects,
        actor: Vec3::new(actor_x, 0.0, 0.0),
      },
    );

    if tick == 0 || *alife.get_last_switch() != before {
      swept.push(tick);
    }
  }

  swept
}

#[test]
fn sweeps_at_the_first_step_then_once_an_update_interval() {
  let mut objects: ObjectRegistry = registry(vec![object(100.0, ObjectSwitching::ByDistance)]);
  let mut alife: Alife = Alife::new(AlifeSettings::default(), Some(LEVEL));

  assert_eq!(run(&mut alife, &mut objects, 0, 1, 0.0), vec![0]);
  assert!(objects.is_online(ObjectId(0)));

  // The actor walks away at once, but the object stays online until the next update, 265 ms later: 27 ticks of 10 ms.
  run(&mut alife, &mut objects, 1, 26, 1_000.0);
  assert!(objects.is_online(ObjectId(0)));

  run(&mut alife, &mut objects, 27, 1, 1_000.0);
  assert!(!objects.is_online(ObjectId(0)));
}

#[test]
fn updates_offline_objects_once_an_update_interval_after_the_sweep() {
  let mut objects: ObjectRegistry = registry(vec![object(1_000.0, ObjectSwitching::ByDistance); 4]);
  let settings: AlifeSettings = AlifeSettings {
    objects_per_update: 1,
    ..AlifeSettings::default()
  };
  let mut alife: Alife = Alife::new(settings, Some(LEVEL));

  for id in 0..4 {
    schedule(&mut objects, id);
  }

  // Updates at ticks 0, 27 and 54: one object each, in id order.
  run(&mut alife, &mut objects, 0, 60, 0.0);

  let stamps: Vec<Option<u64>> = (0..4)
    .map(|id| objects.get_schedules().get(ObjectId(id)).and_then(|row| row.updated))
    .collect();

  assert_eq!(stamps, vec![Some(0), Some(27), Some(54), None]);
  assert_eq!(alife.get_last_schedule().updated, 1);
}

#[test]
fn rests_the_switch_while_the_actor_is_on_no_level_the_graph_holds_but_not_offline_life() {
  let mut objects: ObjectRegistry = registry(vec![object(1.0, ObjectSwitching::ByDistance)]);
  let mut alife: Alife = Alife::new(AlifeSettings::default(), None);

  schedule(&mut objects, 0);
  run(&mut alife, &mut objects, 0, 100, 0.0);

  assert!(objects.get_online().is_empty());
  assert!(
    objects
      .get_schedules()
      .get(ObjectId(0))
      .is_some_and(|row| row.updated.is_some())
  );
}
