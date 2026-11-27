use std::sync::Arc;

use xrf_game_hooks::{EventBuffer, HookRegistry};
use xrf_game_objects::{ObjectCreature, ObjectId, ObjectPlace, ObjectRegistry, ObjectSwitching};
use xrf_math::glam::Vec3;

use crate::events::alife_event::AlifeEvent;
use crate::hooks::alife_hooks::{AlifeHooks, AlifeSwitchGuard};
use crate::settings::alife_settings::AlifeSettings;
use crate::switch::alife_switch::AlifeSwitch;
use crate::switch::alife_switch_direction::AlifeSwitchDirection;
use crate::switch::alife_switch_report::AlifeSwitchReport;
use crate::tests::fixtures::{LEVEL, OTHER_LEVEL, id, object, registry};

fn sweep_with(objects: &mut ObjectRegistry, hooks: &HookRegistry, actor_x: f32) -> Vec<AlifeEvent> {
  let mut events: EventBuffer<AlifeEvent> = EventBuffer::default();

  AlifeSwitch::sweep(
    objects,
    hooks,
    &mut events,
    LEVEL,
    Vec3::new(actor_x, 0.0, 0.0),
    &AlifeSettings::default(),
    1,
  )
  .expect("the switch's hooks");

  events.drain().collect()
}

fn sweep(objects: &mut ObjectRegistry, actor_x: f32) -> AlifeSwitchReport {
  AlifeSwitch::sweep(
    objects,
    &AlifeHooks::of_defaults().expect("A-Life's hooks"),
    &mut EventBuffer::default(),
    LEVEL,
    Vec3::new(actor_x, 0.0, 0.0),
    &AlifeSettings::default(),
    1,
  )
  .expect("the switch's hooks")
}

fn switched(index: u32, direction: AlifeSwitchDirection) -> AlifeEvent {
  AlifeEvent::Switched {
    id: id(index),
    direction,
  }
}

#[test]
fn switches_a_squad_by_its_nearest_member_with_all_its_members() {
  // The squad's own record stands far off; its members at 100 m and 300 m.
  let mut objects: ObjectRegistry = registry(vec![
    object(5_000.0, ObjectSwitching::Group),
    object(100.0, ObjectSwitching::ByDistance),
    object(300.0, ObjectSwitching::ByDistance),
    object(4_000.0, ObjectSwitching::Group),
  ]);

  objects.join_group(id(0), id(1));
  objects.join_group(id(0), id(2));

  let report: AlifeSwitchReport = sweep(&mut objects, 0.0);

  // Members are weighed through their squad, an empty squad not at all.
  assert_eq!(report.swept, 1);
  assert_eq!(report.went_online, 1);
  assert!(objects.is_online(id(0)) && objects.is_online(id(1)) && objects.is_online(id(2)));
  assert!(!objects.is_online(id(3)));

  // 150 m back: the nearer member is 250 m away, beyond the offline distance, the other 450 m.
  sweep(&mut objects, -150.0);
  assert!(objects.get_online().is_empty());
}

#[test]
fn goes_online_within_135_metres_and_offline_beyond_165_keeping_either_state_between() {
  let mut objects: ObjectRegistry = registry(vec![
    object(134.0, ObjectSwitching::ByDistance),
    object(150.0, ObjectSwitching::ByDistance),
    object(166.0, ObjectSwitching::ByDistance),
  ]);
  let report: AlifeSwitchReport = sweep(&mut objects, 0.0);

  assert_eq!(report.swept, 3);
  assert_eq!(report.went_online, 1);
  assert!(objects.is_online(id(0)));
  assert!(!objects.is_online(id(1)) && !objects.is_online(id(2)));

  // The actor walks 20 m closer: 114, 130 and 146 m away.
  sweep(&mut objects, 20.0);
  assert!(objects.is_online(id(0)) && objects.is_online(id(1)));
  assert!(!objects.is_online(id(2)));

  // Back where it started: 150 m is between the two, so the one online stays online.
  sweep(&mut objects, 0.0);
  assert!(objects.is_online(id(1)));

  // 40 m further back: 174 m, beyond the offline distance.
  let report: AlifeSwitchReport = sweep(&mut objects, -24.0);

  assert!(!objects.is_online(id(1)));
  assert_eq!(report.went_offline, 1);
}

#[test]
fn keeps_restrictors_online_and_unswitchable_objects_offline_wherever_they_stand() {
  let mut objects: ObjectRegistry = registry(vec![
    object(5_000.0, ObjectSwitching::AlwaysOnline),
    object(1.0, ObjectSwitching::AlwaysOffline),
    object(1.0, ObjectSwitching::Never),
    object(1.0, ObjectSwitching::Group),
  ]);

  sweep(&mut objects, 0.0);

  assert!(objects.is_online(id(0)));
  assert!(!objects.is_online(id(1)) && !objects.is_online(id(2)) && !objects.is_online(id(3)));
}

#[test]
fn brings_a_dead_creature_online_wherever_it_lies_and_never_takes_it_offline() {
  let mut objects: ObjectRegistry = registry(vec![object(5_000.0, ObjectSwitching::ByDistance)]);

  objects.set_creature(id(0), ObjectCreature { health: 0.0 });

  sweep(&mut objects, 0.0);
  assert!(objects.is_online(id(0)));

  sweep(&mut objects, -10_000.0);
  assert!(objects.is_online(id(0)));
}

#[test]
fn weighs_only_the_current_level_and_what_nothing_holds_which_goes_with_its_holder() {
  let mut objects: ObjectRegistry = registry(vec![
    object(10.0, ObjectSwitching::ByDistance),
    object(1.0, ObjectSwitching::ByDistance),
    object(10_000.0, ObjectSwitching::ByDistance),
  ]);

  objects.move_to(
    id(1),
    ObjectPlace {
      position: Vec3::X,
      level: Some(OTHER_LEVEL),
      game_vertex: Some(0),
      level_vertex: Some(0),
    },
  );
  objects.set_parent(id(2), Some(id(0)));

  let report: AlifeSwitchReport = sweep(&mut objects, 0.0);

  assert_eq!(report.swept, 1);
  assert!(objects.is_online(id(0)));
  assert!(!objects.is_online(id(1)));
  // Held 10 km away, it is online with its holder.
  assert!(objects.is_online(id(2)));
}

#[test]
fn tells_each_object_that_switches_in_id_order_with_what_goes_with_it() {
  let mut objects: ObjectRegistry = registry(vec![
    object(10.0, ObjectSwitching::ByDistance),
    object(500.0, ObjectSwitching::ByDistance),
    object(20.0, ObjectSwitching::ByDistance),
    object(10.0, ObjectSwitching::ByDistance),
  ]);
  let hooks: HookRegistry = AlifeHooks::of_defaults().expect("A-Life's hooks");

  // Object 3 holds object 1, which comes online with it, told between 0 and 2.
  objects.set_parent(id(1), Some(id(3)));

  assert_eq!(
    sweep_with(&mut objects, &hooks, 0.0),
    vec![
      switched(0, AlifeSwitchDirection::Online),
      switched(1, AlifeSwitchDirection::Online),
      switched(2, AlifeSwitchDirection::Online),
      switched(3, AlifeSwitchDirection::Online),
    ]
  );
  assert_eq!(sweep_with(&mut objects, &hooks, 0.0), Vec::new());
  assert_eq!(sweep_with(&mut objects, &hooks, -1_000.0).len(), 4);
}

#[test]
fn asks_the_guards_a_script_put_in_place_of_the_defaults() {
  let mut objects: ObjectRegistry = registry(vec![
    object(10.0, ObjectSwitching::ByDistance),
    object(20.0, ObjectSwitching::ByDistance),
  ]);
  let mut hooks: HookRegistry = AlifeHooks::of_defaults().expect("A-Life's hooks");
  let default: Arc<AlifeSwitchGuard> = hooks
    .replace(
      AlifeHooks::CAN_SWITCH_ONLINE.name,
      Arc::new(|_: &ObjectRegistry, _: ObjectId| false) as Arc<AlifeSwitchGuard>,
    )
    .expect("the default to replace");

  // Object 1 may not go online, the rest as the default says.
  hooks
    .replace(
      AlifeHooks::CAN_SWITCH_ONLINE.name,
      Arc::new(move |objects: &ObjectRegistry, object: ObjectId| object != id(1) && default(objects, object))
        as Arc<AlifeSwitchGuard>,
    )
    .expect("the override to wrap");

  assert_eq!(
    sweep_with(&mut objects, &hooks, 0.0),
    vec![switched(0, AlifeSwitchDirection::Online)]
  );
  assert!(!objects.is_online(id(1)));
}
