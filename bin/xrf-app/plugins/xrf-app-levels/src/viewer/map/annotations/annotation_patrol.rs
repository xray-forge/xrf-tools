use xrf_spawn::Patrol;

use crate::viewer::map::annotations::{AnnotationOutline, AnnotationPoint};

/// A patrol path from above: its points, and the links between them by index.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnotationPatrol {
  pub name: String,
  pub points: Vec<AnnotationPoint>,
  pub links: Vec<(usize, usize)>,
}

impl AnnotationPatrol {
  /// `patrol` from above, its links by the points' places, those reaching past its points left out.
  pub fn of(patrol: &Patrol) -> Self {
    Self {
      name: patrol.name.clone(),
      points: patrol
        .points
        .iter()
        .map(|point| AnnotationOutline::to_point(&point.position))
        .collect(),
      links: patrol
        .links
        .iter()
        .flat_map(|link| {
          link
            .links
            .iter()
            .map(move |(to, _)| (link.index as usize, *to as usize))
        })
        .filter(|(from, to)| *from < patrol.points.len() && *to < patrol.points.len())
        .collect(),
    }
  }
}
