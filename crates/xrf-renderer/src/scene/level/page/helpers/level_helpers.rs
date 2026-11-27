use std::collections::HashMap;
use std::f32::consts::TAU;

use xrf_math::glam::{Mat4, Vec3, Vec4};

use crate::host::level::helpers::render_helper_object::RenderHelperObject;
use crate::host::level::helpers::render_helper_shape::RenderHelperShape;
use crate::host::level::helpers::render_level_helpers::RenderLevelHelpers;
use crate::host::world::render_motion_pose::RenderMotionPose;
use crate::scene::level::page::helpers::level_helper_icon::LevelHelperIcon;
use crate::scene::level::page::helpers::level_helper_motion::LevelHelperMotion;
use crate::scene::level::page::helpers::level_helper_object::LevelHelperObject;
use crate::scene::static_scene::item::object::static_object_box::StaticObjectBox;

/// A level's helpers as wires and icons: each helped object, its segments and the icons it stands in renderer space,
/// built once a set is read and stood again where an object motion carries one, which every view of the scene draws
/// from.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct LevelHelpers {
  /// Moved on with every set read, so a view builds its wires again.
  pub version: u64,
  /// Moved on with every frame a motion moved a carried object, so a view builds the carried objects' wires again.
  pub moved: u64,
  /// By id, ascending.
  pub objects: Vec<LevelHelperObject>,
  /// Every wire's two ends.
  pub segments: Vec<[Vec3; 2]>,
  /// Every icon's point and picture.
  pub icons: Vec<LevelHelperIcon>,
  /// The objects a motion carries.
  carried: Vec<LevelHelperMotion>,
}

impl LevelHelpers {
  /// Segments a sphere's great circle is drawn with: round enough at the size a restrictor fills a view.
  const CIRCLE_SEGMENTS: usize = 32;

  /// The wires and icons of `helpers`, numbered `version`.
  pub(crate) fn new(helpers: RenderLevelHelpers, version: u64) -> Self {
    let mut segments: Vec<[Vec3; 2]> = Vec::new();
    let mut icons: Vec<LevelHelperIcon> = Vec::new();
    let mut built: Vec<(LevelHelperObject, Option<LevelHelperMotion>)> = helpers
      .objects
      .into_iter()
      .map(|object: RenderHelperObject| {
        let transform: Mat4 = Mat4::from_cols_array(&object.transform);
        let start: usize = segments.len();
        let first_icon: usize = icons.len();

        Self::stand(transform, &object.shapes, &mut segments, &mut icons);

        let place: Vec3 = transform.transform_point3(Vec3::ZERO);
        let helped: LevelHelperObject = LevelHelperObject {
          id: object.id,
          kind: object.kind,
          color: object.color,
          draw_distance: object.draw_distance,
          place,
          segments: start..segments.len(),
          icons: first_icon..icons.len(),
          model: object.shapes.iter().find_map(|shape| match shape {
            RenderHelperShape::Model { object } => Some(*object),
            _ => None,
          }),
          is_carried: object.motion.is_some(),
        };
        let carried: Option<LevelHelperMotion> = object.motion.map(|motion| LevelHelperMotion {
          object: 0,
          motion,
          axes: Mat4 {
            w_axis: Vec4::W,
            ..transform
          },
          shapes: object.shapes,
          place,
        });

        (helped, carried)
      })
      .collect();

    // Several objects may share an id, as a campfire's two sounds do, so each carried one is numbered by its place once
    // sorted, not found by its id.
    built.sort_by_key(|(object, _)| object.id);

    let mut carried: Vec<LevelHelperMotion> = Vec::new();
    let objects: Vec<LevelHelperObject> = built
      .into_iter()
      .enumerate()
      .map(|(index, (object, motion))| {
        carried.extend(motion.map(|motion| LevelHelperMotion {
          object: index,
          ..motion
        }));
        object
      })
      .collect();

    Self {
      version,
      moved: 0,
      objects,
      segments,
      icons,
      carried,
    }
  }

  /// The helped object of `id`, if the scene has it.
  pub(crate) fn find(&self, id: u32) -> Option<&LevelHelperObject> {
    self
      .objects
      .binary_search_by_key(&id, |object| object.id)
      .ok()
      .map(|index| &self.objects[index])
  }

  /// Stands each carried object where its motion has its object this frame, in renderer space, its own axes kept, and
  /// moves `moved` on where one moved. One whose motion is still read stays where it stood.
  pub(crate) fn follow(&mut self, motions: &HashMap<String, RenderMotionPose>) {
    let Self {
      moved,
      objects,
      segments,
      icons,
      carried,
      ..
    } = self;
    let mut is_moved: bool = false;
    let mut wires: Vec<[Vec3; 2]> = Vec::new();
    let mut points: Vec<LevelHelperIcon> = Vec::new();

    for carried in carried.iter_mut() {
      let Some(pose) = motions.get(&carried.motion) else {
        continue;
      };
      let at: Vec3 = pose.transform.w_axis.truncate();
      // Helpers stand in renderer space, which mirrors the engine's z.
      let place: Vec3 = Vec3::new(at.x, at.y, -at.z);

      if place == carried.place {
        continue;
      }

      let object: &mut LevelHelperObject = &mut objects[carried.object];

      wires.clear();
      points.clear();
      Self::stand(
        Mat4::from_translation(place) * carried.axes,
        &carried.shapes,
        &mut wires,
        &mut points,
      );
      segments[object.segments.clone()].copy_from_slice(&wires);
      icons[object.icons.clone()].copy_from_slice(&points);
      object.place = place;
      carried.place = place;
      is_moved = true;
    }

    if is_moved {
      *moved += 1;
    }
  }

  /// Adds the wires and icons of `shapes` where `transform` stands them.
  fn stand(
    transform: Mat4,
    shapes: &[RenderHelperShape],
    segments: &mut Vec<[Vec3; 2]>,
    icons: &mut Vec<LevelHelperIcon>,
  ) {
    for shape in shapes {
      match shape {
        RenderHelperShape::Icon { center, icon } => icons.push(LevelHelperIcon {
          center: transform.transform_point3(Vec3::from_array(*center)),
          icon: *icon,
        }),
        shape => Self::add_shape(segments, transform, shape),
      }
    }
  }

  /// A shape's wires where `transform` stands it: a sphere's three great circles, a box's twelve edges, segments as
  /// they are, and none for a model or an icon.
  fn add_shape(segments: &mut Vec<[Vec3; 2]>, transform: Mat4, shape: &RenderHelperShape) {
    match shape {
      RenderHelperShape::Sphere { center, radius } => {
        let center: Vec3 = Vec3::from_array(*center);
        let planes: [(Vec3, Vec3); 3] = [(Vec3::X, Vec3::Y), (Vec3::Y, Vec3::Z), (Vec3::Z, Vec3::X)];

        for (u, v) in planes {
          let point = |step: usize| -> Vec3 {
            let angle: f32 = TAU * step as f32 / Self::CIRCLE_SEGMENTS as f32;

            transform.transform_point3(center + (u * angle.cos() + v * angle.sin()) * *radius)
          };

          segments.extend((0..Self::CIRCLE_SEGMENTS).map(|step| [point(step), point(step + 1)]));
        }
      }
      RenderHelperShape::Box {
        axes: [i, j, k],
        center,
      } => {
        let [i, j, k, center] = [i, j, k, center].map(|it| Vec3::from_array(*it));
        let corner = |index: usize| -> Vec3 {
          let [x, y, z] = [index & 1, (index >> 1) & 1, (index >> 2) & 1].map(|bit| bit as f32 - 0.5);

          transform.transform_point3(center + i * x + j * y + k * z)
        };

        segments.extend(
          StaticObjectBox::EDGES
            .iter()
            .map(|(from, to)| [corner(*from), corner(*to)]),
        );
      }
      // A model's own wires are its box, which only its hovering or selection draws, where the scene stands it; an
      // icon is drawn as itself.
      RenderHelperShape::Model { .. } | RenderHelperShape::Icon { .. } => {}
      RenderHelperShape::Segments { ends } => {
        segments.extend(
          ends
            .iter()
            .map(|ends| ends.map(|end| transform.transform_point3(Vec3::from_array(end)))),
        );
      }
    }
  }
}
