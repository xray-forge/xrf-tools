use glam::{Mat4, Vec3};

use crate::contract::render_helper_options::RenderHelperOptions;
use crate::host::render_helper_object::RenderHelperObject;
use crate::host::render_helper_shape::RenderHelperShape;
use crate::host::render_level_helpers::RenderLevelHelpers;
use crate::scene::level::feature::helpers_feature::HelpersFeature;
use crate::scene::level::helpers::helper_vertex::HelperVertex;
use crate::scene::level::helpers::level_helpers::LevelHelpers;

fn new_object(id: u32, kind: u32, at: Vec3, shapes: Vec<RenderHelperShape>) -> RenderHelperObject {
  RenderHelperObject {
    id,
    kind,
    color: [1.0, 0.5, 0.0],
    draw_distance: 100.0,
    transform: Mat4::from_translation(at).to_cols_array(),
    shapes,
  }
}

/// A sphere of radius two at the origin of the object, which stands at `(10, 0, 0)`, and a cube of two at its origin.
fn new_helpers() -> LevelHelpers {
  LevelHelpers::new(
    RenderLevelHelpers {
      objects: vec![
        new_object(
          7,
          10,
          Vec3::new(10.0, 0.0, 0.0),
          vec![RenderHelperShape::Sphere {
            center: [0.0; 3],
            radius: 2.0,
          }],
        ),
        new_object(
          3,
          11,
          Vec3::ZERO,
          vec![RenderHelperShape::Box {
            axes: [[2.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 2.0]],
            center: [0.0; 3],
          }],
        ),
      ],
    },
    1,
  )
}

fn options(kinds: u32) -> RenderHelperOptions {
  RenderHelperOptions {
    is_shown: true,
    kinds,
    ..RenderHelperOptions::default()
  }
}

#[test]
fn wires_a_sphere_as_three_great_circles_and_a_box_by_its_edges_where_each_stands() {
  let helpers: LevelHelpers = new_helpers();
  let sphere = helpers.find(7).expect("the sphere's object");
  let cube = helpers.find(3).expect("the cube's object");

  assert_eq!(sphere.segments.len(), 96);
  assert_eq!(cube.segments.len(), 12);
  assert_eq!(sphere.place, Vec3::new(10.0, 0.0, 0.0));
  assert!(
    helpers.segments[sphere.segments.clone()]
      .iter()
      .flatten()
      .all(|end| (end.distance(Vec3::new(10.0, 0.0, 0.0)) - 2.0).abs() < 1e-4)
  );
  assert!(
    helpers.segments[cube.segments.clone()]
      .iter()
      .flatten()
      .all(|end| end.abs() == Vec3::ONE)
  );
}

#[test]
fn wires_segments_as_they_stand_and_a_marker_as_three_crossed_axes() {
  let helpers: LevelHelpers = LevelHelpers::new(
    RenderLevelHelpers {
      objects: vec![new_object(
        1,
        0,
        Vec3::new(0.0, 5.0, 0.0),
        vec![
          RenderHelperShape::Segments {
            ends: vec![[[0.0; 3], [1.0, 0.0, 0.0]]],
          },
          RenderHelperShape::Marker {
            center: [0.0; 3],
            size: 2.0,
          },
        ],
      )],
    },
    1,
  );

  assert_eq!(
    helpers.segments,
    vec![
      [Vec3::new(0.0, 5.0, 0.0), Vec3::new(1.0, 5.0, 0.0)],
      [Vec3::new(-1.0, 5.0, 0.0), Vec3::new(1.0, 5.0, 0.0)],
      [Vec3::new(0.0, 4.0, 0.0), Vec3::new(0.0, 6.0, 0.0)],
      [Vec3::new(0.0, 5.0, -1.0), Vec3::new(0.0, 5.0, 1.0)],
    ]
  );
}

#[test]
fn draws_the_kinds_switched_on_less_the_hidden_and_marks_the_selected_unfaded() {
  let helpers: LevelHelpers = new_helpers();
  let ids = |vertices: Vec<HelperVertex>| -> Vec<u32> {
    let mut ids: Vec<u32> = vertices.iter().map(|it| it.object).collect();

    ids.dedup();
    ids
  };

  assert_eq!(
    ids(HelpersFeature::to_vertices(&helpers, &options(1 << 10 | 1 << 11))),
    vec![3, 7]
  );
  assert_eq!(ids(HelpersFeature::to_vertices(&helpers, &options(1 << 11))), vec![3]);
  assert!(
    HelpersFeature::to_vertices(
      &helpers,
      &RenderHelperOptions {
        is_shown: false,
        ..options(u32::MAX)
      }
    )
    .is_empty()
  );
  assert_eq!(
    ids(HelpersFeature::to_vertices(
      &helpers,
      &RenderHelperOptions {
        hidden: vec![3],
        ..options(u32::MAX)
      }
    )),
    vec![7]
  );

  let selected: Vec<HelperVertex> = HelpersFeature::to_vertices(
    &helpers,
    &RenderHelperOptions {
      selected: vec![7],
      distance_scale: 2.0,
      ..options(u32::MAX)
    },
  );

  assert!(
    selected
      .iter()
      .filter(|it| it.object == 7)
      .all(|it| it.color[3] == 1.0 && it.fade[0] == 0.0)
  );
  assert!(
    selected
      .iter()
      .filter(|it| it.object == 3)
      .all(|it| it.color[3] == 0.0 && it.fade[0] == 200.0)
  );
}
