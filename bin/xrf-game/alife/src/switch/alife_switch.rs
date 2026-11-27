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
/// offline; what an object holds goes with it; a squad goes by its nearest member and takes its members with it, a
/// squad with none staying as it is (`CSE_ALifeOnlineOfflineGroup`).
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
      // What an object holds, and a squad's member, switches with its holder or squad.
      if record.level != Some(level) || record.parent.is_some() || objects.get_members().contains(id) {
        continue;
      }

      let distance: f32 = match record.switching {
        // A squad stands as near as its nearest member: online when any is within the online distance, offline when
        // all are beyond the offline one (`CSE_ALifeOnlineOfflineGroup::try_switch_online`, `try_switch_offline`).
        ObjectSwitching::Group => {
          let members: &[ObjectId] = objects.get_group_members(id);

          if members.is_empty() {
            continue;
          }

          members
            .iter()
            .filter_map(|member| objects.get_records().get(*member))
            .map(|member| actor.distance(member.position))
            .fold(f32::INFINITY, f32::min)
        }
        _ => actor.distance(record.position),
      };

      report.swept += 1;

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
      (ObjectSwitching::Never, _) => None,
      (ObjectSwitching::AlwaysOnline, online) => (!online).then_some(true),
      (ObjectSwitching::AlwaysOffline, online) => online.then_some(false),
      (ObjectSwitching::ByDistance | ObjectSwitching::Group, false) => {
        (!may_go_offline || is_within_online).then_some(true)
      }
      (ObjectSwitching::ByDistance | ObjectSwitching::Group, true) => {
        (may_go_offline && is_beyond_offline).then_some(false)
      }
    }
  }
}
