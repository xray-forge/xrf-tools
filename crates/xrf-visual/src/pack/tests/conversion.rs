//! Holds the handedness contract: what mirroring Z does to a vector, a uv pair, and a box.

use xrf_ogf::{OgfBox, OgfSphere};

use crate::data::visual_bounds::VisualBounds;
use crate::pack::tests::fixtures::vector;
use crate::pack::visual_conversion::{
  convert_declared_bounds, convert_texture_coordinates, convert_vector, reverse_triangle_winding,
};

#[test]
fn mirrors_z_and_leaves_the_other_axes_alone() {
  assert_eq!(convert_vector(&vector(1.0, 2.0, 3.0)), vector(1.0, 2.0, -3.0));
}

#[test]
fn leaves_texture_coordinates_as_the_file_stores_them() {
  // A compressed texture cannot be flipped on upload, so its rows stay top first and Direct3D V already samples them
  // correctly. Flipping here would render every texture upside down.
  assert_eq!(convert_texture_coordinates(0.25, 0.75), (0.25, 0.75));
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
