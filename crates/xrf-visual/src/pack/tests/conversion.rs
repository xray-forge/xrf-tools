//! Holds the handedness contract: what mirroring Z does to a vector, a uv pair, and a box.

use xrf_math::{Matrix4x4, Vector3d};
use xrf_ogf::{OgfBox, OgfSphere};

use crate::data::visual_bounds::VisualBounds;
use crate::pack::tests::fixtures::vector;
use crate::pack::visual_conversion::{
  convert_declared_bounds, convert_placement, convert_uvs, convert_vector, reverse_triangle_winding,
};

#[test]
fn mirrors_z_and_leaves_the_other_axes_alone() {
  assert_eq!(convert_vector(&vector(1.0, 2.0, 3.0)), vector(1.0, 2.0, -3.0));
}

#[test]
fn leaves_uvs_as_the_file_stores_them() {
  // A compressed texture cannot be flipped on upload, so its rows stay top first and Direct3D V already samples them
  // correctly. Flipping here would render every texture upside down.
  assert_eq!(convert_uvs(0.25, 0.75), (0.25, 0.75));
}

#[test]
fn reorders_a_declared_box_after_mirroring_z() {
  // Mirroring Z swaps which face is nearest, so a box converted corner by corner would come out with
  // its minimum Z above its maximum and read as empty.
  let bounds: VisualBounds = convert_declared_bounds(
    &OgfBox {
      min: vector(-1.0, -2.0, -3.0),
      max: vector(4.0, 5.0, 6.0),
    },
    &OgfSphere {
      position: vector(1.0, 2.0, 3.0),
      radius: 7.0,
    },
  );

  assert_eq!(bounds.bounding_box.min, vector(-1.0, -2.0, -6.0));
  assert_eq!(bounds.bounding_box.max, vector(4.0, 5.0, 3.0));
  assert!(bounds.bounding_box.min.z <= bounds.bounding_box.max.z);
}

#[test]
fn mirrors_a_declared_sphere_centre_and_keeps_its_radius() {
  let bounds: VisualBounds = convert_declared_bounds(
    &OgfBox {
      min: vector(0.0, 0.0, 0.0),
      max: vector(1.0, 1.0, 1.0),
    },
    &OgfSphere {
      position: vector(1.0, 2.0, 3.0),
      radius: 7.0,
    },
  );

  assert_eq!(bounds.bounding_sphere.center, vector(1.0, 2.0, -3.0));
  assert_eq!(bounds.bounding_sphere.radius, 7.0);
}

#[test]
fn swaps_the_second_and_third_index_of_every_triangle() {
  let mut indices: Vec<u16> = vec![0, 1, 2, 3, 4, 5];

  reverse_triangle_winding(&mut indices);

  assert_eq!(indices, vec![0, 2, 1, 3, 5, 4]);
}

#[test]
fn keeps_every_triangle_at_its_own_offset() {
  // Reversing the array would give each triangle the same winding while moving all of them, which
  // silently invalidates every detail table offset. Winding must be a local swap.
  let mut indices: Vec<u16> = vec![0, 1, 2, 3, 4, 5];

  reverse_triangle_winding(&mut indices);

  assert_eq!(&indices[0..3], &[0, 2, 1], "expect the first triangle to stay first");
  assert_eq!(&indices[3..6], &[3, 5, 4], "expect the second triangle to stay second");
}

// A placed vertex has to land in the same place whether it is placed and then converted, or converted and then placed
// by the converted transform. Getting this wrong mirrors every tree of a level about the level's own z axis, which
// looks plausible until it is compared with the game.
#[test]
fn places_a_converted_vertex_where_converting_a_placed_one_lands() {
  let placement: Matrix4x4 = Matrix4x4 {
    values: [
      0.0, 0.0, 1.0, 0.0, //
      0.0, 2.0, 0.0, 0.0, //
      -1.0, 0.0, 0.0, 0.0, //
      10.0, 20.0, 30.0, 1.0,
    ],
  };

  let local: Vector3d = vector(1.0, 2.0, 3.0);

  let placed_then_converted: Vector3d = convert_vector(&placement.transform_point(&local));
  let converted_then_placed: Vector3d = convert_placement(&placement).transform_point(&convert_vector(&local));

  assert_eq!(placed_then_converted, converted_then_placed);
}

#[test]
fn leaves_a_placement_of_no_rotation_alone_except_where_it_moves_along_z() {
  let placement: Matrix4x4 = Matrix4x4 {
    values: [
      1.0, 0.0, 0.0, 0.0, //
      0.0, 1.0, 0.0, 0.0, //
      0.0, 0.0, 1.0, 0.0, //
      5.0, 6.0, 7.0, 1.0,
    ],
  };

  let converted: Matrix4x4 = convert_placement(&placement);

  assert_eq!(converted.get_translation(), vector(5.0, 6.0, -7.0));
}
