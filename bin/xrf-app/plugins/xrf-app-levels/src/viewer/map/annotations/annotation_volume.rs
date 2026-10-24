use xrf_spawn::{AlifeObject, AlifeObjectInherited, SpawnLevelGraph};
use xrf_visual::VisualTransform;

use crate::viewer::map::annotations::{AnnotationOutline, AnnotationPoint, LevelAnnotationKind};

/// One logic volume of the level seen from above: a smart terrain, restrictor, anomaly or level changer.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationVolume {
  pub kind: LevelAnnotationKind,
  pub name: String,
  pub outlines: Vec<AnnotationOutline>,
  /// Whether it keeps the actor inside it, as a restrictor of the in type does.
  pub is_in: bool,
  /// A level changer's destination: the level's name, and the way toward it from this level across the world the
  /// levels share, engine `x` and `z`, where the graph places both.
  pub destination: Option<(String, Option<AnnotationPoint>)>,
}

impl AnnotationVolume {
  /// `RestrictionSpace::eDefaultRestrictorTypeIn` and `eRestrictorTypeIn`: a restrictor keeping the actor inside it.
  const RESTRICTOR_TYPES_IN: [u8; 2] = [2, 4];

  /// `object`'s logic volume from above on the level named `level`, or `None` for an object that is none.
  pub fn of(object: &AlifeObject, graph: &SpawnLevelGraph, level: &str) -> Option<Self> {
    let (kind, shapes, restrictor_type, destination) = match &object.inherited {
      AlifeObjectInherited::SeSmartTerrain(terrain) => {
        let restrictor = &terrain.base.base;

        (
          LevelAnnotationKind::SmartTerrains,
          &restrictor.shape,
          restrictor.restrictor_type,
          None,
        )
      }
      AlifeObjectInherited::CseAlifeSpaceRestrictor(restrictor) => (
        LevelAnnotationKind::Restrictors,
        &restrictor.shape,
        restrictor.restrictor_type,
        None,
      ),
      AlifeObjectInherited::CseAlifeAnomalousZone(zone) => {
        let restrictor = &zone.base.base.base;

        (
          LevelAnnotationKind::Anomalies,
          &restrictor.shape,
          restrictor.restrictor_type,
          None,
        )
      }
      AlifeObjectInherited::CseAlifeTorridZone(zone) => {
        let restrictor = &zone.base.base;

        (
          LevelAnnotationKind::Anomalies,
          &restrictor.shape,
          restrictor.restrictor_type,
          None,
        )
      }
      AlifeObjectInherited::SeLevelChanger(changer) => (
        LevelAnnotationKind::LevelChangers,
        &changer.base.shape,
        changer.base.restrictor_type,
        Some((
          changer.dest_level_name.clone(),
          Self::get_toward(graph, level, &changer.dest_level_name),
        )),
      ),
      _ => return None,
    };
    // `CSpaceRestrictor::net_Spawn`: the shapes stand in the object's own space, its `XFORM` at its place and angle.
    let to_world: VisualTransform = VisualTransform::of_spawn(&object.position, &object.direction).mirrored();

    Some(Self {
      kind,
      name: object.name.clone(),
      outlines: shapes
        .iter()
        .map(|shape| AnnotationOutline::of(shape, &to_world))
        .collect(),
      is_in: Self::RESTRICTOR_TYPES_IN.contains(&restrictor_type),
      destination,
    })
  }

  /// The way from `level` toward `destination` across the world the graph places every level in, as a unit, or `None`
  /// where the graph lacks either or places them together.
  fn get_toward(graph: &SpawnLevelGraph, level: &str, destination: &str) -> Option<AnnotationPoint> {
    let (from, to) = (&graph.get_level(level)?.offset, &graph.get_level(destination)?.offset);
    let (x, z): (f32, f32) = (to.x - from.x, to.z - from.z);
    let length: f32 = x.hypot(z);

    (length > f32::EPSILON).then(|| [x / length, z / length])
  }
}
