use xrf_spawn::{AlifeObject, AlifeObjectInherited};

/// Whether an object stands on the AI map, `CSE_ALifeObject::used_ai_locations` and its overrides: such an object takes
/// the node under it and that node's game vertex, any other the level's first vertex.
pub(crate) struct SpawnAiLocations;

impl SpawnAiLocations {
  /// `flUsedAI_Locations`, the flag the classes that do not decide for themselves read.
  const FLAG: u32 = 1 << 7;

  pub(crate) fn is_used(object: &AlifeObject) -> bool {
    match &object.inherited {
      // Every class on `CSE_ALifeSpaceRestrictor`, and the classes whose override says no.
      AlifeObjectInherited::CseAlifeSpaceRestrictor(_)
      | AlifeObjectInherited::SeLevelChanger(_)
      | AlifeObjectInherited::CseAlifeTeamBaseZone(_)
      | AlifeObjectInherited::CseAlifeAnomalousZone(_)
      | AlifeObjectInherited::CseAlifeTorridZone(_)
      | AlifeObjectInherited::CseAlifeZoneVisual(_)
      | AlifeObjectInherited::CseAlifeObjectPhysic(_)
      | AlifeObjectInherited::CseAlifeObjectHangingLamp(_)
      | AlifeObjectInherited::CseAlifeObjectProjector(_)
      | AlifeObjectInherited::CseAlifeHelicopter(_)
      | AlifeObjectInherited::CseAlifeCar(_)
      | AlifeObjectInherited::CseAlifeObjectBreakable(_)
      | AlifeObjectInherited::CseAlifeObjectClimable(_)
      | AlifeObjectInherited::CseAlifeItemBolt(_)
      | AlifeObjectInherited::CseAlifeCreatureCrow(_)
      | AlifeObjectInherited::CseAlifeCreaturePhantom(_)
      | AlifeObjectInherited::CseAlifePhSkeletonObject(_)
      | AlifeObjectInherited::CseAlifeGraphPoint(_) => false,
      // Smart zones, smart covers and creatures, whose overrides say yes.
      AlifeObjectInherited::SeSmartTerrain(_)
      | AlifeObjectInherited::ClearSkySeSmartTerrain(_)
      | AlifeObjectInherited::ClearSkySeSimFaction(_)
      | AlifeObjectInherited::SeSmartCover(_)
      | AlifeObjectInherited::SeActor(_)
      | AlifeObjectInherited::SeStalker(_)
      | AlifeObjectInherited::SeMonster(_)
      | AlifeObjectInherited::ClearSkySeActor(_)
      | AlifeObjectInherited::ClearSkySeStalker(_)
      | AlifeObjectInherited::ClearSkySeMonster(_) => true,
      other => other.get_abstract().is_some_and(|base| base.flags & Self::FLAG != 0),
    }
  }
}
