use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_game_hooks::{EventBuffer, HookRegistry};
use xrf_game_objects::{ObjectId, ObjectRegistry, ObjectSwitching};
use xrf_math::glam::Vec3;

use crate::events::alife_event::AlifeEvent;
use crate::hooks::alife_hooks::{AlifeHooks, AlifeSwitchGuard};
use crate::settings::alife_settings::AlifeSettings;
use crate::switch::alife_switch_direction::AlifeSwitchDirection;
use crate::switch::alife_switch_report::AlifeSwitchReport;

/// Which objects are online: X-Ray's switch (`CALifeSwitchManager`, `xrGame/alife_switch_manager.cpp`), kept to its
/// rules and nothing of its stalls.
///
/// Kept: one sweep weighs every object of the current level that nothing holds, in id order (the registry's level
/// index); an offline one goes online within the online distance of the actor, an online one offline beyond the
/// offline distance (the gap between them keeps an object at the edge from flapping); one that may go online but not
/// offline goes online wherever it stands, one that may go offline but not online goes offline, one that may do
/// neither stays as it is; what an object holds goes with it; a squad goes by its nearest member and takes its members
/// with it, a squad with none staying as it is (`CSE_ALifeOnlineOfflineGroup`). Whether an object may go online or
/// offline is asked of the hook registry (`alife.can_switch_online`, `alife.can_switch_offline`), and each object that
/// switches is told as an event (`alife.on_switched`).
///
/// Not carried over: X-Ray builds the whole client object as it switches one online (`Process_spawn`, on the main
/// thread), which stalls a frame when many come at once. Here going online is a row in the registry's online table;
/// what an online object needs is the business of the systems that read the table, off this step.
pub struct AlifeSwitch;

impl AlifeSwitch {
  /// Sweeps the objects on `level` once around the `actor`, in the engine's space, at `tick`, telling each object that
  /// switches to `events`.
  ///
  /// # Errors
  ///
  /// Returns a not found error where a guard of the switch is not registered.
  pub fn sweep(
    objects: &mut ObjectRegistry,
    hooks: &HookRegistry,
    events: &mut EventBuffer<AlifeEvent>,
    level: u8,
    actor: Vec3,
    settings: &AlifeSettings,
    tick: u64,
  ) -> XrfResult<AlifeSwitchReport> {
    let can_switch_online: Arc<AlifeSwitchGuard> = hooks.get(&AlifeHooks::CAN_SWITCH_ONLINE)?;
    let can_switch_offline: Arc<AlifeSwitchGuard> = hooks.get(&AlifeHooks::CAN_SWITCH_OFFLINE)?;
    let online_distance: f32 = settings.get_online_distance();
    let offline_distance: f32 = settings.get_offline_distance();
    let mut report: AlifeSwitchReport = AlifeSwitchReport::default();
    let mut going_online: Vec<ObjectId> = Vec::new();
    let mut going_offline: Vec<ObjectId> = Vec::new();

    for id in objects.get_level_objects(level) {
      // What an object holds, and a squad's member, switches with its holder or squad.
      if objects.get_parent(id).is_some() || objects.get_members().contains(id) {
        continue;
      }

      let Some(record) = objects.get_records().get(id) else {
        continue;
      };
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

      let decision: Option<AlifeSwitchDirection> = Self::decide(
        objects.is_online(id),
        can_switch_online(objects, id),
        can_switch_offline(objects, id),
        distance <= online_distance,
        distance > offline_distance,
      );

      match decision {
        Some(AlifeSwitchDirection::Online) => going_online.push(id),
        Some(AlifeSwitchDirection::Offline) => going_offline.push(id),
        None => {}
      }
    }

    report.went_online = going_online.len();
    report.went_offline = going_offline.len();

    for id in going_online {
      for switched in objects.bring_online(id, tick) {
        events.push(AlifeEvent::Switched {
          id: switched,
          direction: AlifeSwitchDirection::Online,
        });
      }
    }

    for id in going_offline {
      for switched in objects.take_offline(id) {
        events.push(AlifeEvent::Switched {
          id: switched,
          direction: AlifeSwitchDirection::Offline,
        });
      }
    }

    Ok(report)
  }

  /// Which way an object switches, none where it stays as it is, by whether it is online, whether it may go online
  /// and offline as it stands, and where it stands against the two distances (`CALifeSwitchManager::switch_object`).
  fn decide(
    is_online: bool,
    can_switch_online: bool,
    can_switch_offline: bool,
    is_within_online: bool,
    is_beyond_offline: bool,
  ) -> Option<AlifeSwitchDirection> {
    if is_online {
      (can_switch_offline && (!can_switch_online || is_beyond_offline)).then_some(AlifeSwitchDirection::Offline)
    } else {
      (can_switch_online && (!can_switch_offline || is_within_online)).then_some(AlifeSwitchDirection::Online)
    }
  }
}
