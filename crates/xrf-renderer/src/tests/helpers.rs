use glam::{Mat4, Vec3};

use crate::contract::render_helper_icon::RenderHelperIcon;
use crate::contract::render_helper_options::RenderHelperOptions;
use crate::host::render_helper_object::RenderHelperObject;
use crate::host::render_helper_shape::RenderHelperShape;
use crate::host::render_level_helpers::RenderLevelHelpers;
use crate::scene::level::feature::helpers_feature::HelpersFeature;
use crate::scene::level::helpers::gpu::helper_icon_instance::HelperIconInstance;
use crate::scene::level::helpers::gpu::helper_vertex::HelperVertex;
use crate::scene::level::helpers::level_helpers::LevelHelpers;
use crate::scene::level::helpers::view::helper_part::HelperPart;
use crate::scene::static_scene::static_object_box::StaticObjectBox;

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

/// A scene standing no spawned object.
fn no_boxes(_: u32) -> Option<StaticObjectBox> {
  None
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
fn wires_segments_where_their_object_stands_them() {
  let helpers: LevelHelpers = LevelHelpers::new(
    RenderLevelHelpers {
      objects: vec![new_object(
        1,
        0,
        Vec3::new(0.0, 5.0, 0.0),
        vec![RenderHelperShape::Segments {
          ends: vec![[[0.0; 3], [1.0, 0.0, 0.0]], [[0.0; 3], [0.0, 0.0, -2.0]]],
        }],
      )],
    },
    1,
  );

  assert_eq!(
    helpers.segments,
    vec![
      [Vec3::new(0.0, 5.0, 0.0), Vec3::new(1.0, 5.0, 0.0)],
      [Vec3::new(0.0, 5.0, 0.0), Vec3::new(0.0, 5.0, -2.0)],
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
    ids(HelpersFeature::to_vertices(
      &helpers,
      &options(1 << 10 | 1 << 11),
      HelperPart::Every,
      no_boxes
    )),
    vec![3, 7]
  );
  assert_eq!(
    ids(HelpersFeature::to_vertices(
      &helpers,
      &options(1 << 11),
      HelperPart::Every,
      no_boxes
    )),
    vec![3]
  );
  assert!(
    HelpersFeature::to_vertices(
      &helpers,
      &RenderHelperOptions {
        is_shown: false,
        ..options(u32::MAX)
      },
      HelperPart::Every,
      no_boxes
    )
    .is_empty()
  );
  assert_eq!(
    ids(HelpersFeature::to_vertices(
      &helpers,
      &RenderHelperOptions {
        hidden: vec![3],
        ..options(u32::MAX)
      },
      HelperPart::Every,
      no_boxes
    )),
    vec![7]
  );

  // Every kept object's draw as none were singled out; the selected one's are drawn apart, over them.
  let selecting: RenderHelperOptions = RenderHelperOptions {
    selected: vec![7],
    distance_scale: 2.0,
    ..options(u32::MAX)
  };
  let every: Vec<HelperVertex> = HelpersFeature::to_vertices(&helpers, &selecting, HelperPart::Every, no_boxes);
  let singled: Vec<HelperVertex> = HelpersFeature::to_vertices(&helpers, &selecting, HelperPart::Singled, no_boxes);

  assert_eq!(ids(every.clone()), vec![3, 7]);
  assert!(every.iter().all(|it| it.color[3] == 0.0 && it.fade[0] == 200.0));
  assert_eq!(ids(singled.clone()), vec![7]);
  assert!(singled.iter().all(|it| it.color[3] == 1.0 && it.fade[0] == 0.0));
}

// A model marks its place in its colour while it stands in the scene, by place, and draws its box only while hovered or
// selected, its hovering lifting it half as far as a selection.
#[test]
fn marks_each_standing_model_by_its_place_and_boxes_the_hovered_and_selected() {
  let helpers: LevelHelpers = LevelHelpers::new(
    RenderLevelHelpers {
      objects: vec![
        new_object(4, 2, Vec3::ZERO, vec![RenderHelperShape::Model { object: 40 }]),
        new_object(5, 2, Vec3::ZERO, vec![RenderHelperShape::Model { object: 50 }]),
        new_object(6, 2, Vec3::ZERO, vec![RenderHelperShape::Model { object: 60 }]),
      ],
    },
    1,
  );
  // The second model stands at a lower place than the first, and the third stands nowhere yet.
  let places = |object: u32| -> Option<u32> {
    match object {
      40 => Some(900),
      50 => Some(300),
      _ => None,
    }
  };
  let boxes = |_: u32| -> Option<StaticObjectBox> {
    Some(StaticObjectBox {
      transform: Mat4::IDENTITY,
      min: Vec3::ZERO,
      max: Vec3::ONE,
    })
  };
  let orange: u32 = 255 | 128 << 8 | 255 << 24;

  assert_eq!(helpers.segments.len(), 0);
  assert_eq!(
    HelpersFeature::to_marks(&helpers, &options(1 << 2), places),
    vec![[300, orange], [900, orange]]
  );
  assert!(HelpersFeature::to_marks(&helpers, &options(1 << 3), places).is_empty());
  assert!(HelpersFeature::to_vertices(&helpers, &options(1 << 2), HelperPart::Every, boxes).is_empty());

  let singled: Vec<HelperVertex> = HelpersFeature::to_vertices(
    &helpers,
    &RenderHelperOptions {
      selected: vec![4],
      hovered: Some(5),
      ..options(1 << 2)
    },
    HelperPart::Singled,
    boxes,
  );

  // Two ends of twelve edges for each of the two.
  assert_eq!(singled.len(), 48);
  assert!(singled.iter().all(|it| it.color[3]
    == if it.object == 4 {
      HelperVertex::SELECTED
    } else {
      HelperVertex::HOVERED
    }));
  assert!(
    singled
      .iter()
      .all(|it| Vec3::from_array(it.position).cmple(Vec3::ONE).all())
  );
}

// An icon stands where its object stands it and draws as its object's wires would, by its cell of the atlas.
#[test]
fn stands_each_icon_at_its_point_by_its_cell_drawn_as_its_object() {
  let helpers: LevelHelpers = LevelHelpers::new(
    RenderLevelHelpers {
      objects: vec![
        new_object(
          8,
          4,
          Vec3::new(1.0, 2.0, 3.0),
          vec![RenderHelperShape::Icon {
            center: [0.0, 1.0, 0.0],
            icon: RenderHelperIcon::Light,
          }],
        ),
        new_object(
          9,
          5,
          Vec3::ZERO,
          vec![RenderHelperShape::Icon {
            center: [0.0; 3],
            icon: RenderHelperIcon::Sound,
          }],
        ),
      ],
    },
    1,
  );
  let icons: Vec<HelperIconInstance> = HelpersFeature::to_icons(
    &helpers,
    &RenderHelperOptions {
      selected: vec![8],
      ..options(1 << 4)
    },
    HelperPart::Singled,
  );

  assert_eq!(helpers.segments.len(), 0);
  assert_eq!(icons.len(), 1);
  assert_eq!(icons[0].position, [1.0, 3.0, 3.0]);
  assert_eq!(icons[0].icon, RenderHelperIcon::Light.get_index());
  assert_eq!(icons[0].object, 8);
  assert_eq!(icons[0].color[3], HelperVertex::SELECTED);
  assert_eq!(icons[0].fade[0], 0.0);
}
