use xrf_spawn::SpawnLevelGraph;

use crate::objects::{LevelObject, LevelObjectKind, LevelObjectVolume, LevelRestriction};
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
  /// `object`'s logic volume from above on the level named `level`, or `None` for an object the map draws none of.
  pub fn of(object: &LevelObject, graph: &SpawnLevelGraph, level: &str) -> Option<Self> {
    let kind: LevelAnnotationKind = match object.kind {
      LevelObjectKind::SmartTerrains => LevelAnnotationKind::SmartTerrains,
      LevelObjectKind::Restrictors => LevelAnnotationKind::Restrictors,
      LevelObjectKind::Anomalies => LevelAnnotationKind::Anomalies,
      LevelObjectKind::LevelChangers => LevelAnnotationKind::LevelChangers,
      _ => return None,
    };
    let volume: &LevelObjectVolume = object.volume.as_ref()?;

    Some(Self {
      kind,
      name: object.name.clone(),
      outlines: volume
        .shapes
        .iter()
        .map(|shape| AnnotationOutline::of(shape, &object.transform))
        .collect(),
      is_in: volume.restriction == LevelRestriction::In,
      destination: volume
        .destination
        .as_ref()
        .map(|destination| (destination.clone(), Self::get_toward(graph, level, destination))),
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
