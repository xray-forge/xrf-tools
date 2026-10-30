use crate::objects::{LevelObject, LevelObjectPatrol};
use crate::viewer::map::annotations::{AnnotationOutline, AnnotationPoint};

/// A patrol path from above: its points, and the links between them by index.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationPatrol {
  pub name: String,
  pub points: Vec<AnnotationPoint>,
  pub links: Vec<(usize, usize)>,
}

impl AnnotationPatrol {
  /// `object`'s path from above, or `None` for an object that is no patrol path.
  pub fn of(object: &LevelObject) -> Option<Self> {
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
