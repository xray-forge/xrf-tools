use xrf_spawn::{AlifeClass, AlifeObject};

/// How an object goes online and offline, as X-Ray decides it (`CSE_ALifeObject::can_switch_online` and
/// `can_switch_offline`, `xrServerEntities/xrServer_Objects_ALife.cpp`; `CSE_ALifeDynamicObject::try_switch_online` and
/// `try_switch_offline`, `xrGame/alife_dynamic_object.cpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectSwitching {
  /// Online near the actor, offline far from it: both flags set.
  ByDistance,
  /// Always online: it may not go offline (its flag unset, or a space restrictor, a breakable, a climable or a smart
  /// cover, which refuse whatever their flags say). A dead creature and an empty ammo box stay online too, which the
  /// switch reads from the record as they change.
  AlwaysOnline,
  /// Always offline: it may not go online.
  AlwaysOffline,
  /// A group (a squad), which goes online and offline as its members do (`CSE_ALifeOnlineOfflineGroup`).
  Group,
  /// Not an object that switches: a graph point.
  Never,
}

impl ObjectSwitching {
  /// `flSwitchOnline`.
  const FLAG_ONLINE: u32 = 1 << 1;

  /// `flSwitchOffline`.
  const FLAG_OFFLINE: u32 = 1 << 2;

  /// The flags an object created during a game holds: every one set, as `CSE_ALifeObject`'s constructor sets them.
  pub const CREATED_FLAGS: u32 = u32::MAX;

  /// How a spawned object switches, from its class and flags.
  pub fn of_spawn(object: &AlifeObject) -> Self {
    match object.inherited.get_abstract() {
      Some(placed) => Self::of_class(object.inherited.get_class(), placed.flags),
      None => Self::Never,
    }
  }

  /// How an object of `class` with `flags` switches.
  pub fn of_class(class: AlifeClass, flags: u32) -> Self {
    match class {
      AlifeClass::SimSquadScripted => return Self::Group,
      // `CSE_SmartCover` answers online yes and offline no, flags aside.
      AlifeClass::SeSmartCover => return Self::AlwaysOnline,
      _ => {}
    }

    let can_online: bool = flags & Self::FLAG_ONLINE != 0;
    let can_offline: bool = flags & Self::FLAG_OFFLINE != 0 && !Self::refuses_offline(class);

    match (can_online, can_offline) {
      (false, _) => Self::AlwaysOffline,
      (true, false) => Self::AlwaysOnline,
      (true, true) => Self::ByDistance,
    }
  }

  /// The classes whose `can_switch_offline` answers false whatever the flags: the space restrictors (zones, smart
  /// terrains, level changers), breakables and climables.
  fn refuses_offline(class: AlifeClass) -> bool {
    matches!(
      class,
      AlifeClass::CseAlifeSpaceRestrictor
        | AlifeClass::CseAlifeAnomalousZone
        | AlifeClass::CseAlifeTorridZone
        | AlifeClass::CseAlifeZoneVisual
        | AlifeClass::CseAlifeTeamBaseZone
        | AlifeClass::SeSmartTerrain
        | AlifeClass::ClearSkySeSmartTerrain
        | AlifeClass::SeLevelChanger
        | AlifeClass::CseAlifeObjectBreakable
        | AlifeClass::CseAlifeObjectClimable
    )
  }
}
