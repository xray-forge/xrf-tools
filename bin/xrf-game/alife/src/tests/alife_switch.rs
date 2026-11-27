use xrf_game_objects::{ObjectCreature, ObjectId, ObjectRegistry, ObjectSwitching};
use xrf_math::glam::Vec3;

use crate::settings::alife_settings::AlifeSettings;
use crate::switch::alife_switch::AlifeSwitch;
use crate::switch::alife_switch_report::AlifeSwitchReport;
use crate::tests::fixtures::{LEVEL, OTHER_LEVEL, object, registry};

fn sweep(objects: &mut ObjectRegistry, actor_x: f32) -> AlifeSwitchReport {
  AlifeSwitch::sweep(
    objects,
    LEVEL,
    Vec3::new(actor_x, 0.0, 0.0),
    &AlifeSettings::default(),
    1,
  )
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
  assert!(objects.is_online(ObjectId(0)));
  assert!(!objects.is_online(ObjectId(1)) && !objects.is_online(ObjectId(2)));

  // The actor walks 20 m closer: 114, 130 and 146 m away.
  sweep(&mut objects, 20.0);
  assert!(objects.is_online(ObjectId(0)) && objects.is_online(ObjectId(1)));
  assert!(!objects.is_online(ObjectId(2)));

  // Back where it started: 150 m is between the two, so the one online stays online.
  sweep(&mut objects, 0.0);
  assert!(objects.is_online(ObjectId(1)));

  // 40 m further back: 174 m, beyond the offline distance.
  let report: AlifeSwitchReport = sweep(&mut objects, -24.0);

  assert!(!objects.is_online(ObjectId(1)));
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

  assert!(objects.is_online(ObjectId(0)));
  assert!(!objects.is_online(ObjectId(1)) && !objects.is_online(ObjectId(2)) && !objects.is_online(ObjectId(3)));
}

#[test]
fn brings_a_dead_creature_online_wherever_it_lies_and_never_takes_it_offline() {
  let mut objects: ObjectRegistry = registry(vec![object(5_000.0, ObjectSwitching::ByDistance)]);

  objects
    .get_creatures_mut()
    .insert(ObjectId(0), ObjectCreature { health: 0.0 });

  sweep(&mut objects, 0.0);
  assert!(objects.is_online(ObjectId(0)));

  sweep(&mut objects, -10_000.0);
  assert!(objects.is_online(ObjectId(0)));
}

#[test]
fn weighs_only_the_current_level_and_what_nothing_holds_which_goes_with_its_holder() {
  let mut elsewhere = object(1.0, ObjectSwitching::ByDistance);
  let mut held = object(10_000.0, ObjectSwitching::ByDistance);

  elsewhere.level = Some(OTHER_LEVEL);
  held.parent = Some(ObjectId(0));

  let mut objects: ObjectRegistry = registry(vec![object(10.0, ObjectSwitching::ByDistance), elsewhere, held]);
  let report: AlifeSwitchReport = sweep(&mut objects, 0.0);

  assert_eq!(report.swept, 1);
  assert!(objects.is_online(ObjectId(0)));
  assert!(!objects.is_online(ObjectId(1)));
  // Held 10 km away, it is online with its holder.
  assert!(objects.is_online(ObjectId(2)));
}
