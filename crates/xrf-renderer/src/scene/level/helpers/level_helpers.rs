use std::f32::consts::TAU;

use glam::{Mat4, Vec3};

use crate::host::render_helper_object::RenderHelperObject;
use crate::host::render_helper_shape::RenderHelperShape;
use crate::host::render_level_helpers::RenderLevelHelpers;
use crate::scene::level::helpers::level_helper_icon::LevelHelperIcon;
use crate::scene::level::helpers::level_helper_object::LevelHelperObject;
use crate::scene::static_scene::static_object_box::StaticObjectBox;

/// A level's helpers as wires and icons: each helped object, its segments and the icons it stands in renderer space,
/// built once a set is read, which every view of the scene draws from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelHelpers {
  /// Moved on with every set read, so a view builds its wires again.
  pub version: u64,
  /// By id, ascending.
  pub objects: Vec<LevelHelperObject>,
  /// Every wire's two ends.
  pub segments: Vec<[Vec3; 2]>,
  /// Every icon's point and picture.
  pub icons: Vec<LevelHelperIcon>,
}

impl LevelHelpers {
  /// Segments a sphere's great circle is drawn with: round enough at the size a restrictor fills a view.
  const CIRCLE_SEGMENTS: usize = 32;

  /// The wires and icons of `helpers`, numbered `version`.
  pub fn new(helpers: RenderLevelHelpers, version: u64) -> Self {
    let mut segments: Vec<[Vec3; 2]> = Vec::new();
    let mut icons: Vec<LevelHelperIcon> = Vec::new();
    let mut objects: Vec<LevelHelperObject> = helpers
      .objects
      .into_iter()
      .map(|object: RenderHelperObject| {
        let transform: Mat4 = Mat4::from_cols_array(&object.transform);
        let start: usize = segments.len();
        let first_icon: usize = icons.len();

        for shape in &object.shapes {
          match shape {
            RenderHelperShape::Icon { center, icon } => icons.push(LevelHelperIcon {
              center: transform.transform_point3(Vec3::from_array(*center)),
              icon: *icon,
            }),
            shape => Self::add_shape(&mut segments, transform, shape),
          }
        }

        LevelHelperObject {
          id: object.id,
          kind: object.kind,
          color: object.color,
          draw_distance: object.draw_distance,
          place: transform.transform_point3(Vec3::ZERO),
          segments: start..segments.len(),
          icons: first_icon..icons.len(),
          model: object.shapes.iter().find_map(|shape| match shape {
            RenderHelperShape::Model { object } => Some(*object),
            _ => None,
          }),
        }
      })
      .collect();

    objects.sort_by_key(|object| object.id);

    Self {
      version,
      objects,
      segments,
      icons,
    }
  }

  /// The helped object of `id`, if the scene has it.
  pub fn find(&self, id: u32) -> Option<&LevelHelperObject> {
    self
      .objects
      .binary_search_by_key(&id, |object| object.id)
      .ok()
      .map(|index| &self.objects[index])
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
          let [x, y, z] = [index & 1, index >> 1 & 1, index >> 2 & 1].map(|bit| bit as f32 - 0.5);

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
