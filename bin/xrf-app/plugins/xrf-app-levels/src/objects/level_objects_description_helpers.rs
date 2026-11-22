use xrf_math::glam::Vec3;
use xrf_renderer::{RenderHelperObject, RenderHelperShape, RenderLevelHelpers};

use crate::helpers::HelperLinkLines;
use crate::objects::{LevelObject, LevelObjectId, LevelObjectKindEntry, LevelObjectShape, LevelObjectsDescription};

/// What each level object is drawn as among the helpers.
struct HelperLook;

impl HelperLook {
  /// What `object` is drawn as, in its own space; none for an object drawing no helper.
  fn list_shapes(object: &LevelObject) -> Vec<RenderHelperShape> {
    if let (LevelObjectId::Spawn { index }, Some(spawn)) = (object.id, &object.spawn)
      && object.kind.get_entry().is_drawn
      && spawn.visual.is_some()
    {
      return vec![RenderHelperShape::Model { object: index }];
    }

    if let Some(shapes) = object.get_shapes() {
      return shapes.iter().map(RenderHelperShape::from).collect();
    }

    if let Some(patrol) = &object.patrol {
      let origin: Vec3 = Vec3::from(&object.transform.c);
      let point = |index: u32| -> Option<Vec3> { patrol.points.get(index as usize).map(|it| Vec3::from(it) - origin) };
      let ends: Vec<[[f32; 3]; 2]> = patrol
        .links
        .iter()
        .filter_map(|link| Some(HelperLinkLines::list(point(link.from)?, point(link.to)?)))
        .flatten()
        .collect();

      return vec![RenderHelperShape::Segments { ends }];
    }

    // What stands at a point with no shape of its own draws its kind's icon there.
    object.kind.get_entry().icon.map_or_else(Vec::new, |icon| {
      vec![RenderHelperShape::Icon { center: [0.0; 3], icon }]
    })
  }
}

impl From<&LevelObjectsDescription> for RenderLevelHelpers {
  /// The level objects the renderer draws helpers for, each by its place among the listing's objects, switched by its
  /// kind's place in the kind table, in its kind's colour: a drawn spawned object by its model, the volumes and the
  /// placements covering a space by their shapes, a patrol path by its links, and what stands at a point by its kind's
  /// icon.
  fn from(description: &LevelObjectsDescription) -> Self {
    Self {
      objects: description
        .objects
        .iter()
        .enumerate()
        .filter_map(|(index, object): (usize, &LevelObject)| {
          let shapes: Vec<RenderHelperShape> = HelperLook::list_shapes(object);
          let entry: &LevelObjectKindEntry = object.kind.get_entry();

          (!shapes.is_empty()).then(|| RenderHelperObject {
            id: index as u32,
            kind: object.kind as u32,
            color: entry.get_rgb(),
            draw_distance: entry.draw_distance,
            transform: object.transform.to_matrix(),
            shapes,
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
