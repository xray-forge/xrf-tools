use xrf_spawn::AlifeClass;

/// What kind of object A-Life updates while it is offline, as X-Ray's classes that are `CSE_ALifeSchedulable` decide
/// whether one needs updating (`need_update`, `xrServerEntities/xrServer_Objects_ALife.cpp` and
/// `xrServer_Objects_ALife_Monsters.cpp`; `CSE_ALifeOnlineOfflineGroup::need_update`, `xrGame/alife_online_offline_group.cpp`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectScheduleKind {
  /// A stalker or a monster (`CSE_ALifeMonsterAbstract`): updated while it is offline, under direct control and alive.
  Creature,
  /// A smart terrain, or Clear Sky's faction (`CSE_ALifeSmartZone`): updated while it is offline and under direct
  /// control.
  SmartZone,
  /// A squad (`CSE_ALifeOnlineOfflineGroup`): updated while it is offline, its members through it.
  Group,
}

impl ObjectScheduleKind {
  /// The kind of an object of `class`, none for a class A-Life never updates offline (the actor, a crow or a phantom,
  /// which are creatures but no monsters, and every item, zone and physical object).
  pub fn of_class(class: AlifeClass) -> Option<Self> {
    match class {
      AlifeClass::SeStalker | AlifeClass::SeMonster | AlifeClass::ClearSkySeStalker | AlifeClass::ClearSkySeMonster => {
        Some(Self::Creature)
      }
      AlifeClass::SeSmartTerrain | AlifeClass::ClearSkySeSmartTerrain | AlifeClass::ClearSkySeSimFaction => {
        Some(Self::SmartZone)
      }
      AlifeClass::SimSquadScripted => Some(Self::Group),
      _ => None,
    }
  }
}
