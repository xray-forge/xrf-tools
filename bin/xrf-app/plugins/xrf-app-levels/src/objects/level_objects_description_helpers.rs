use xrf_renderer::{RenderHelperObject, RenderHelperShape, RenderLevelHelpers};

use crate::objects::{LevelObject, LevelObjectId, LevelObjectKindEntry, LevelObjectShape, LevelObjectsDescription};

impl From<&LevelObjectsDescription> for RenderLevelHelpers {
  /// The volumes among the level's objects as the renderer draws their helpers: each by its spawn index, switched by
  /// its kind's place in the kind table, in its kind's colour.
  // todo: Hand over the point kinds as icons once the renderer draws them.
  fn from(description: &LevelObjectsDescription) -> Self {
    Self {
      objects: description
        .objects
        .iter()
        .filter_map(|object: &LevelObject| {
          let volume = object.volume.as_ref()?;
          let LevelObjectId::Spawn { index } = object.id;
          let entry: &LevelObjectKindEntry = object.kind.get_entry();

          Some(RenderHelperObject {
            id: index,
            kind: object.kind as u32,
            color: entry.get_rgb(),
            draw_distance: entry.draw_distance,
            transform: object.transform.to_matrix(),
            shapes: volume.shapes.iter().map(RenderHelperShape::from).collect(),
          })
        })
        .collect(),
    }
  }
}

impl From<&LevelObjectShape> for RenderHelperShape {
  fn from(shape: &LevelObjectShape) -> Self {
    match shape {
      LevelObjectShape::Sphere { center, radius } => Self::Sphere {
        center: [center.x, center.y, center.z],
        radius: *radius,
      },
      LevelObjectShape::Box { axes, center } => Self::Box {
        axes: axes.each_ref().map(|axis| [axis.x, axis.y, axis.z]),
        center: [center.x, center.y, center.z],
      },
    }
  }
}
