use crate::objects::{LevelObject, LevelObjectId, LevelObjectKind};
use crate::viewer::export::map::annotations::{AnnotationOutline, AnnotationPoint};

/// A level object standing at a point, drawn from above as a dot.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnnotationMarker {
  pub kind: LevelObjectKind,
  pub name: String,
  pub point: AnnotationPoint,
  /// Text drawn beside it even when objects are not named: a graph point's game vertex id.
  pub tag: Option<String>,
}

impl AnnotationMarker {
  /// `object` as a dot; `None` for a volume, a patrol path, or a part drawn with its parent, such as a patrol point.
  pub(crate) fn of(object: &LevelObject) -> Option<Self> {
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
