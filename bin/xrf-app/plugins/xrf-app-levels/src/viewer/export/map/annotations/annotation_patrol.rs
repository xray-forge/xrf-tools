use crate::objects::{LevelObject, LevelObjectPatrol};
use crate::viewer::export::map::annotations::{AnnotationOutline, AnnotationPoint};

/// A patrol path from above: its points, and links between them by index.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AnnotationPatrol {
  pub name: String,
  pub points: Vec<AnnotationPoint>,
  pub links: Vec<(usize, usize)>,
}

impl AnnotationPatrol {
  /// `object`'s path from above; `None` unless it is a patrol path.
  pub(crate) fn of(object: &LevelObject) -> Option<Self> {
    let patrol: &LevelObjectPatrol = object.patrol.as_ref()?;

    Some(Self {
      name: object.name.clone(),
      points: patrol
        .points
        .iter()
        .map(AnnotationOutline::from_renderer_point)
        .collect(),
      links: patrol
        .links
        .iter()
        .map(|link| (link.from as usize, link.to as usize))
        .collect(),
    })
  }
}
