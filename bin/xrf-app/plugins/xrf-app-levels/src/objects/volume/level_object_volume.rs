use serde::Serialize;
use xrf_spawn::{AlifeObjectInherited, AlifeObjectSpaceRestrictor};

use crate::objects::{LevelObjectShape, LevelRestriction};

/// What a logic volume covers and does: its shapes, whom it restricts, and the level a changer leads to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectVolume {
  /// In the object's own space, renderer space; none for a volume the spawn gives no shape.
  pub shapes: Vec<LevelObjectShape>,
  pub restriction: LevelRestriction,
  /// The level a level changer leads to, by name.
  pub destination: Option<String>,
}

impl LevelObjectVolume {
  /// The volume of a spawned object, or `None` for a class that is none: a smart terrain, a space restrictor, an
  /// anomalous or torrid zone, a level changer or a ladder.
  pub fn of(inherited: &AlifeObjectInherited) -> Option<Self> {
    let (restrictor, destination): (&AlifeObjectSpaceRestrictor, Option<&String>) = match inherited {
      AlifeObjectInherited::SeSmartTerrain(terrain) => (&terrain.base.base, None),
      AlifeObjectInherited::CseAlifeSpaceRestrictor(restrictor) => (restrictor, None),
      AlifeObjectInherited::CseAlifeAnomalousZone(zone) => (&zone.base.base.base, None),
      AlifeObjectInherited::CseAlifeTorridZone(zone) => (&zone.base.base, None),
      AlifeObjectInherited::SeLevelChanger(changer) => (&changer.base, Some(&changer.dest_level_name)),
      AlifeObjectInherited::CseAlifeObjectClimable(climable) => {
        return Some(Self {
          shapes: climable.base.shape.iter().map(LevelObjectShape::from).collect(),
          restriction: LevelRestriction::None,
          destination: None,
        });
      }
      _ => return None,
    };

    Some(Self {
      shapes: restrictor.shape.iter().map(LevelObjectShape::from).collect(),
      restriction: restrictor.restrictor_type.into(),
      destination: destination.cloned(),
    })
  }
}
