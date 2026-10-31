use glam::Vec3;
use xrf_renderer::{RenderHelperObject, RenderHelperShape, RenderLevelHelpers};

use crate::objects::{LevelObject, LevelObjectId, LevelObjectKindEntry, LevelObjectShape, LevelObjectsDescription};

/// What each level object is drawn as among the helpers.
struct HelperLook;

impl HelperLook {
  /// An arrowhead's barbs: metres at most, a share of the link at most, and their angle off the link.
  const ARROW_LENGTH: f32 = 0.6;
  const ARROW_SHARE: f32 = 0.25;
  const ARROW_ANGLE: f32 = 0.45;

  /// A link from `from` to `to` as its line and an arrowhead's two barbs at `to`, turned about the vertical; none for
  /// a link of no length.
  fn to_link(from: Vec3, to: Vec3) -> Vec<[[f32; 3]; 2]> {
    let span: Vec3 = to - from;
    let length: f32 = span.length();

    if length <= f32::EPSILON {
      return Vec::new();
    }

    let back: Vec3 = -span / length * (length * Self::ARROW_SHARE).min(Self::ARROW_LENGTH);
    let barb = |angle: f32| -> [[f32; 3]; 2] {
      let (sin, cos): (f32, f32) = angle.sin_cos();

      [
        to.to_array(),
        (to + Vec3::new(back.x * cos - back.z * sin, back.y, back.x * sin + back.z * cos)).to_array(),
      ]
    };

    vec![
      [from.to_array(), to.to_array()],
      barb(Self::ARROW_ANGLE),
      barb(-Self::ARROW_ANGLE),
    ]
  }

  /// What `object` is drawn as, in its own space; none for an object drawing no helper.
  fn to_shapes(object: &LevelObject) -> Vec<RenderHelperShape> {
    if let (LevelObjectId::Spawn { index }, Some(spawn)) = (object.id, &object.spawn)
      && object.kind.get_entry().is_drawn
      && spawn.visual.is_some()
    {
      return vec![RenderHelperShape::Model { object: index }];
    }

    if let Some(volume) = &object.volume {
      return volume.shapes.iter().map(RenderHelperShape::from).collect();
    }

    if let Some(patrol) = &object.patrol {
      let origin: Vec3 = Vec3::new(object.transform.c.x, object.transform.c.y, object.transform.c.z);
      let point = |index: u32| -> Option<Vec3> {
        patrol
          .points
          .get(index as usize)
          .map(|it| Vec3::new(it.x, it.y, it.z) - origin)
      };
      let ends: Vec<[[f32; 3]; 2]> = patrol
        .links
        .iter()
        .filter_map(|link| Some(Self::to_link(point(link.from)?, point(link.to)?)))
        .flatten()
        .collect();

      return vec![RenderHelperShape::Segments { ends }];
    }

    if let Some(placement) = object.placement.as_ref().filter(|it| !it.shapes.is_empty()) {
      return placement.shapes.iter().map(RenderHelperShape::from).collect();
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
          let shapes: Vec<RenderHelperShape> = HelperLook::to_shapes(object);
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
