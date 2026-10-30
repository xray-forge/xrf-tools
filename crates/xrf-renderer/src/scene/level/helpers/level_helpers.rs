use std::f32::consts::TAU;

use glam::{Mat4, Vec3};

use crate::host::render_helper_object::RenderHelperObject;
use crate::host::render_helper_shape::RenderHelperShape;
use crate::host::render_level_helpers::RenderLevelHelpers;
use crate::scene::level::helpers::level_helper_object::LevelHelperObject;

/// A level's helpers as wires: each helped object and its segments in renderer space, built once a set is read, which
/// every view of the scene draws from.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelHelpers {
  /// Moved on with every set read, so a view builds its wires again.
  pub version: u64,
  /// By id, ascending.
  pub objects: Vec<LevelHelperObject>,
  /// Every wire's two ends.
  pub segments: Vec<[Vec3; 2]>,
}

impl LevelHelpers {
  /// Segments a sphere's great circle is drawn with: round enough at the size a restrictor fills a view.
  const CIRCLE_SEGMENTS: usize = 32;

  /// The unit cube's twelve edges, by corners numbered `x + 2y + 4z`.
  const BOX_EDGES: [(usize, usize); 12] = [
    (0, 1),
    (2, 3),
    (4, 5),
    (6, 7),
    (0, 2),
    (1, 3),
    (4, 6),
    (5, 7),
    (0, 4),
    (1, 5),
    (2, 6),
    (3, 7),
  ];

  /// The wires of `helpers`, numbered `version`.
  pub fn new(helpers: RenderLevelHelpers, version: u64) -> Self {
    let mut segments: Vec<[Vec3; 2]> = Vec::new();
    let mut objects: Vec<LevelHelperObject> = helpers
      .objects
      .into_iter()
      .map(|object: RenderHelperObject| {
        let transform: Mat4 = Mat4::from_cols_array(&object.transform);
        let start: usize = segments.len();

        for shape in &object.shapes {
          Self::add_shape(&mut segments, transform, shape);
        }

        LevelHelperObject {
          id: object.id,
          kind: object.kind,
          color: object.color,
          draw_distance: object.draw_distance,
          place: transform.transform_point3(Vec3::ZERO),
          segments: start..segments.len(),
        }
      })
      .collect();

    objects.sort_by_key(|object| object.id);

    Self {
      version,
      objects,
      segments,
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
  /// they are, and a marker's three axes.
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

        segments.extend(Self::BOX_EDGES.iter().map(|(from, to)| [corner(*from), corner(*to)]));
      }
      RenderHelperShape::Segments { ends } => {
        segments.extend(
          ends
            .iter()
            .map(|ends| ends.map(|end| transform.transform_point3(Vec3::from_array(end)))),
        );
      }
      RenderHelperShape::Marker { center, size } => {
        let center: Vec3 = Vec3::from_array(*center);

        segments.extend([Vec3::X, Vec3::Y, Vec3::Z].map(|axis| {
          let reach: Vec3 = axis * *size * 0.5;

          [
            transform.transform_point3(center - reach),
            transform.transform_point3(center + reach),
          ]
        }));
      }
    }
  }
}
