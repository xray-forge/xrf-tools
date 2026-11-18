use xrf_spawn::SpawnLevelGraph;

use crate::objects::{LevelObject, LevelObjectKind, LevelObjectShape, LevelObjectVolume, LevelRestriction};
use crate::viewer::export::map::annotations::{AnnotationOutline, AnnotationPoint};

/// A level object covering a space, seen from above: a logic volume or a placement with shapes.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnnotationVolume {
  pub kind: LevelObjectKind,
  pub name: String,
  pub outlines: Vec<AnnotationOutline>,
  /// Whether it keeps the actor inside, as an in-type restrictor does.
  pub is_in: bool,
  /// A level changer's destination level, and the unit direction toward it in engine `x` and `z` when the graph places
  /// both levels.
  pub destination: Option<(String, Option<AnnotationPoint>)>,
}

impl AnnotationVolume {
  /// `object`'s footprint on the level named `level`; `None` for an object without shapes.
  pub(crate) fn of(object: &LevelObject, graph: &SpawnLevelGraph, level: &str) -> Option<Self> {
    let shapes: &[LevelObjectShape] = object.get_shapes()?;
    let volume: Option<&LevelObjectVolume> = object.volume.as_ref();

    Some(Self {
      kind: object.kind,
      name: object.name.clone(),
      outlines: shapes
        .iter()
        .map(|shape| AnnotationOutline::of(shape, &object.transform))
        .collect(),
      is_in: volume.is_some_and(|volume| volume.restriction == LevelRestriction::In),
      destination: volume
        .and_then(|volume| volume.destination.as_ref())
        .map(|destination| (destination.clone(), Self::get_toward(graph, level, destination))),
    })
  }

  /// The unit direction from `level` toward `destination` in the graph's world; `None` when the graph lacks either or
  /// places them at one point.
  fn get_toward(graph: &SpawnLevelGraph, level: &str, destination: &str) -> Option<AnnotationPoint> {
    let (from, to) = (&graph.get_level(level)?.offset, &graph.get_level(destination)?.offset);
    let (x, z): (f32, f32) = (to.x - from.x, to.z - from.z);
    let length: f32 = x.hypot(z);

    (length > f32::EPSILON).then(|| [x / length, z / length])
  }
}
