use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_game_hooks::{HookKind, HookPoint, HookRegistry};
use xrf_game_objects::{ObjectId, ObjectRegistry, ObjectSwitching};

use crate::events::alife_event::AlifeEvent;

/// A guard of the switch: whether object `id` may go online, or offline, as it stands.
pub type AlifeSwitchGuard = dyn Fn(&ObjectRegistry, ObjectId) -> bool + Send + Sync;

/// A listener to A-Life's events, reading the registry as the step left it.
pub type AlifeListener = dyn Fn(&ObjectRegistry, &AlifeEvent) + Send + Sync;

/// A-Life's hook points, each registered with A-Life's own behaviour, which the switch reads back through the
/// registry every sweep.
pub struct AlifeHooks;

impl AlifeHooks {
  /// Whether an object may go online (`CSE_ALifeObject::can_switch_online`): by default, by how it switches.
  pub const CAN_SWITCH_ONLINE: HookPoint<AlifeSwitchGuard> = HookPoint::new("alife.can_switch_online", HookKind::Guard);

  /// Whether an object may go offline (`can_switch_offline`): by default, by how it switches, and neither a dead
  /// creature nor an empty ammo box (`ObjectRegistry::may_go_offline`).
  pub const CAN_SWITCH_OFFLINE: HookPoint<AlifeSwitchGuard> =
    HookPoint::new("alife.can_switch_offline", HookKind::Guard);

  /// An object went online or offline; by default nothing more happens.
  pub const ON_SWITCHED: HookPoint<AlifeListener> = HookPoint::new("alife.on_switched", HookKind::Event);

  /// Registers A-Life's points with their defaults.
  ///
  /// # Errors
  ///
  /// Returns an invalid error where a point is registered already.
  pub fn register(hooks: &mut HookRegistry) -> XrfResult<()> {
    hooks.register(
      &Self::CAN_SWITCH_ONLINE,
      Arc::new(Self::can_switch_online) as Arc<AlifeSwitchGuard>,
    )?;
    hooks.register(
      &Self::CAN_SWITCH_OFFLINE,
      Arc::new(Self::can_switch_offline) as Arc<AlifeSwitchGuard>,
    )?;
    hooks.register(
      &Self::ON_SWITCHED,
      Arc::new(|_: &ObjectRegistry, _: &AlifeEvent| {}) as Arc<AlifeListener>,
    )
  }

  /// A hook registry holding A-Life's points and nothing else, as a test or a tool steps A-Life alone.
  ///
  /// # Errors
  ///
  /// Returns an invalid error where a point is registered twice, which a fresh registry never holds.
  pub fn of_defaults() -> XrfResult<HookRegistry> {
    let mut hooks: HookRegistry = HookRegistry::default();

    Self::register(&mut hooks)?;

    Ok(hooks)
  }

  fn can_switch_online(objects: &ObjectRegistry, id: ObjectId) -> bool {
    objects.get_records().get(id).is_some_and(|record| {
      matches!(
        record.switching,
        ObjectSwitching::ByDistance | ObjectSwitching::AlwaysOnline | ObjectSwitching::Group
      )
    })
  }

  fn can_switch_offline(objects: &ObjectRegistry, id: ObjectId) -> bool {
    objects.may_go_offline(id)
      && objects.get_records().get(id).is_some_and(|record| {
        matches!(
          record.switching,
          ObjectSwitching::ByDistance | ObjectSwitching::AlwaysOffline | ObjectSwitching::Group
        )
      })
  }
}
