use xrf_game_objects::{ObjectId, ObjectRegistry, ObjectSwitching};
use xrf_math::glam::Vec3;

use crate::settings::alife_settings::AlifeSettings;
use crate::switch::alife_switch_report::AlifeSwitchReport;

/// Which objects are online: X-Ray's switch (`CALifeSwitchManager`, `xrGame/alife_switch_manager.cpp`), kept to its
/// rules and nothing of its stalls.
///
/// Kept: one sweep weighs every object of the current level that nothing holds, in id order; an offline one goes online
/// within the online distance of the actor, an online one offline beyond the offline distance (the gap between them
/// keeps an object at the edge from flapping); an object that may not go offline (a restrictor, a breakable, a climable,
/// a smart cover, a dead creature, an empty ammo box) goes online wherever it stands, one that may not go online stays
/// offline; what an object holds goes with it.
///
/// Not carried over: X-Ray builds the whole client object as it switches one online (`Process_spawn`, on the main
/// thread), which stalls a frame when many come at once. Here going online is a row in the registry's online table;
/// what an online object needs is the business of the systems that read the table, off this step.
pub struct AlifeSwitch;

impl AlifeSwitch {
  /// Sweeps the objects on `level` once around the `actor`, in the engine's space, at `tick`.
  pub fn sweep(
    objects: &mut ObjectRegistry,
    level: u8,
    actor: Vec3,
    settings: &AlifeSettings,
    tick: u64,
  ) -> AlifeSwitchReport {
    let online_distance: f32 = settings.get_online_distance();
    let offline_distance: f32 = settings.get_offline_distance();
    let mut report: AlifeSwitchReport = AlifeSwitchReport::default();
    let mut going_online: Vec<ObjectId> = Vec::new();
    let mut going_offline: Vec<ObjectId> = Vec::new();

    for (id, record) in objects.get_records().iter() {
      if record.level != Some(level) || record.parent.is_some() {
        continue;
      }

      report.swept += 1;

      let distance: f32 = actor.distance(record.position);
      let decision: Option<bool> = Self::decide(
        record.switching,
        objects.is_online(id),
        objects.may_go_offline(id),
        distance <= online_distance,
        distance > offline_distance,
      );

      match decision {
        Some(true) => going_online.push(id),
        Some(false) => going_offline.push(id),
        None => {}
      }
    }

    report.went_online = going_online.len();
    report.went_offline = going_offline.len();

    for id in going_online {
      objects.bring_online(id, tick);
    }

    for id in going_offline {
      objects.take_offline(id);
    }

    report
  }

  /// Whether an object goes online (`Some(true)`), offline (`Some(false)`), or stays as it is (`None`), by how it
  /// switches, whether it is online, whether it may go offline as it stands, and where it stands against the two
  /// distances.
  fn decide(
    switching: ObjectSwitching,
    is_online: bool,
    may_go_offline: bool,
    is_within_online: bool,
    is_beyond_offline: bool,
  ) -> Option<bool> {
    match (switching, is_online) {
      (ObjectSwitching::Never | ObjectSwitching::Group, _) => None,
      (ObjectSwitching::AlwaysOnline, online) => (!online).then_some(true),
      (ObjectSwitching::AlwaysOffline, online) => online.then_some(false),
      (ObjectSwitching::ByDistance, false) => (!may_go_offline || is_within_online).then_some(true),
      (ObjectSwitching::ByDistance, true) => (may_go_offline && is_beyond_offline).then_some(false),
    }
  }
}
