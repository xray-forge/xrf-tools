use crate::objects::{LevelObject, LevelObjectId, LevelObjectKind};
use crate::viewer::map::annotations::{AnnotationOutline, AnnotationPoint};

/// One object of the level standing at a point, seen from above: a dot where it stands.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationMarker {
  pub kind: LevelObjectKind,
  pub name: String,
  pub point: AnnotationPoint,
  /// What is written beside it whether or not every object is named: a graph point's game vertex id.
  pub tag: Option<String>,
}

impl AnnotationMarker {
  /// `object` as a dot, or `None` for an object drawn otherwise: one covering a space, a patrol path, or a part
  /// drawn with what it belongs to, as a patrol's points are.
  pub fn of(object: &LevelObject) -> Option<Self> {
    if object.get_shapes().is_some() || object.patrol.is_some() || object.parent.is_some() {
      return None;
    }

    Some(Self {
      kind: object.kind,
      name: object.name.clone(),
      point: AnnotationOutline::from_renderer_point(&object.transform.c),
      tag: match object.id {
        LevelObjectId::GraphPoint { vertex } => Some(vertex.to_string()),
        _ => None,
      },
    })
  }
}
