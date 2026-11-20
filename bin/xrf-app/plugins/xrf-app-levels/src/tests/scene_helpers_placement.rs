use std::sync::Arc;

use glam::{Mat4, Vec3};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::SessionSnapshot;
use xrf_editable_level::{
  EditableCustomObject, EditableObjectTransform, EditableShape, EditableShapeElement, EditableShapeObject,
  EditableSpawnPointKind, EditableSpawnPointObject, EditableWayLink, EditableWayObject, EditableWayPoint,
};
use xrf_ini::IniSection;
use xrf_math::{Affine3d, Matrix4x4, Vector3d};
use xrf_renderer::{RenderHelperObject, RenderHelperShape, RenderLevelHelpers};
use xrf_visual::convert_vector;
use xrf_world::WorldLevelSource;

use crate::scene::{SceneRenderSource, SceneSource, SelectedScene, ShownScenes};
use crate::tests::fixtures::{new_scenes, open_scene, scene_part, write_scene, write_scene_part};

// The two sides compose in different orders in `f32`.
const TOLERANCE: f32 = 1e-4;

// Turned about every axis and scaled unevenly, so a row-major basis, a flipped angle sign or a one-sided mirror shows.
const TRANSFORM: EditableObjectTransform = EditableObjectTransform {
  position: Vector3d::new(12.5, -3.0, 7.25),
  rotation: Vector3d::new(0.4, -1.1, 0.7),
  scale: Vector3d::new(2.0, 0.5, 1.5),
};

const BOX: Affine3d = Affine3d {
  x_axis: Vector3d::new(1.5, 0.2, 0.0),
  y_axis: Vector3d::new(0.0, 0.8, 0.3),
  z_axis: Vector3d::new(0.1, 0.0, 2.0),
  translation: Vector3d::new(1.0, -2.0, 0.5),
};

const SPHERE_CENTER: Vector3d = Vector3d::new(0.5, 1.0, -1.0);

fn read_helpers(source: &SceneSource) -> RenderLevelHelpers {
  let assets: AssetMountState = AssetMountState::new();
  let opened: Arc<SessionSnapshot<SelectedScene>> = open_scene(&new_scenes(&assets), &assets, source);
  let drawn: Arc<SceneRenderSource> = ShownScenes::default().get_source(opened, &assets);

  drawn.read_helpers().expect("the helpers read")
}

fn stand(object: &RenderHelperObject, local: [f32; 3]) -> Vec3 {
  Mat4::from_cols_array(&object.transform).transform_point3(Vec3::from_array(local))
}

fn convert(point: &Vector3d) -> Vec3 {
  Vec3::from(convert_vector(point))
}

fn assert_near(drawn: Vec3, expected: Vec3) {
  assert!(
    drawn.distance(expected) <= TOLERANCE,
    "drawn at {drawn:?}, placed at {expected:?}"
  );
}

fn list_box_corners(object: &RenderHelperObject, shape: &RenderHelperShape) -> Vec<Vec3> {
  let RenderHelperShape::Box { axes, center } = shape else {
    panic!("{shape:?} is no box");
  };
  let [i, j, k] = axes.map(Vec3::from_array);

  EditableShape::UNIT_BOX_CORNERS
    .iter()
    .map(|[x, y, z]| {
      stand(
        object,
        (Vec3::from_array(*center) + i * *x + j * *y + k * *z).to_array(),
      )
    })
    .collect()
}

#[test]
fn a_shapes_helper_stands_where_its_transform_places_its_elements() {
  let source: SceneSource = write_scene("scene_helpers_placement/shape");

  write_scene_part(
    &source,
    "shape.part",
    scene_part(vec![EditableShapeObject {
      object: EditableCustomObject::new("shape_0000", TRANSFORM),
      shape: EditableShape {
        usage: 0,
        elements: vec![
          EditableShapeElement::Box(BOX),
          EditableShapeElement::Sphere {
            center: SPHERE_CENTER,
            radius: 0.75,
          },
        ],
      },
    }]),
  );

  let helpers: RenderLevelHelpers = read_helpers(&source);
  let object: &RenderHelperObject = &helpers.objects[0];
  let world: Matrix4x4 = TRANSFORM.to_matrix();

  // The editor places a box element's corners by the object's world transform times the element's matrix.
  for (drawn, placed) in list_box_corners(object, &object.shapes[0])
    .into_iter()
    .zip(EditableShape::place_unit_box(&world.multiply(&Matrix4x4::from(&BOX))))
  {
    assert_near(drawn, convert(&placed));
  }

  let RenderHelperShape::Sphere { center, radius } = &object.shapes[1] else {
    panic!("{:?} is no sphere", object.shapes[1]);
  };

  assert_near(stand(object, *center), convert(&world.transform_point(&SPHERE_CENTER)));
  assert_eq!(*radius, 0.75, "scaled by the transform as it is drawn");
}

#[test]
fn an_entitys_volume_stands_at_its_spawn_point_scaled_by_the_shape_attached() {
  let source: SceneSource = write_scene("scene_helpers_placement/spawn");
  let attached: EditableShapeObject = EditableShapeObject {
    object: EditableCustomObject::new(
      "shape_0000",
      EditableObjectTransform {
        scale: Vector3d::new(2.0, 2.0, 2.0),
        ..TRANSFORM
      },
    ),
    shape: EditableShape {
      usage: 0,
      elements: vec![EditableShapeElement::Box(BOX)],
    },
  };
  let elements: Vec<EditableShapeElement> = attached.get_scaled_elements();

  write_scene_part(
    &source,
    "spawn.part",
    scene_part(vec![EditableSpawnPointObject {
      object: EditableCustomObject::new("restrictor_0000", TRANSFORM),
      kind: EditableSpawnPointKind::Entity {
        section: String::from("space_restrictor"),
        flags: 0,
        data: IniSection::new("spawndata"),
      },
      attached: Some(attached),
    }]),
  );

  let helpers: RenderLevelHelpers = read_helpers(&source);
  let object: &RenderHelperObject = &helpers.objects[0];
  // The build gives the entity its spawn point's place and turn, unscaled, and the shape's elements scaled.
  let entity: Matrix4x4 = EditableObjectTransform {
    scale: Vector3d::ONE,
    ..TRANSFORM
  }
  .to_matrix();
  let EditableShapeElement::Box(scaled) = &elements[0] else {
    panic!("the element is a box");
  };

  for (drawn, placed) in list_box_corners(object, &object.shapes[0])
    .into_iter()
    .zip(EditableShape::place_unit_box(
      &entity.multiply(&Matrix4x4::from(scaled)),
    ))
  {
    assert_near(drawn, convert(&placed));
  }
}

#[test]
fn a_ways_links_and_marks_stand_on_its_points() {
  let source: SceneSource = write_scene("scene_helpers_placement/way");
  let positions: [Vector3d; 2] = [Vector3d::new(3.0, 1.0, 9.0), Vector3d::new(-2.0, 0.5, 14.0)];

  write_scene_part(
    &source,
    "way.part",
    scene_part(vec![EditableWayObject {
      object: EditableCustomObject::new("walk", TRANSFORM),
      points: vec![
        EditableWayPoint {
          name: String::from("wp00"),
          position: positions[0],
          flags: 0,
          selected: false,
          links: vec![EditableWayLink {
            to: 1,
            probability: 1.0,
          }],
        },
        EditableWayPoint {
          name: String::from("wp01"),
          position: positions[1],
          flags: 0,
          selected: false,
          links: Vec::new(),
        },
      ],
      kind: EditableWayObject::PATROL_PATH,
    }]),
  );

  let helpers: RenderLevelHelpers = read_helpers(&source);
  let object: &RenderHelperObject = &helpers.objects[0];
  let RenderHelperShape::Segments { ends } = &object.shapes[0] else {
    panic!("{:?} are no segments", object.shapes[0]);
  };

  // A way's points are in world space: its object's own transform moves none of them.
  assert_near(stand(object, ends[0][0]), convert(&positions[0]));
  assert_near(stand(object, ends[0][1]), convert(&positions[1]));

  for (shape, position) in object.shapes[1..].iter().zip(&positions) {
    let RenderHelperShape::Icon { center, .. } = shape else {
      panic!("{shape:?} is no mark");
    };

    assert_near(stand(object, *center), convert(position));
  }
}
