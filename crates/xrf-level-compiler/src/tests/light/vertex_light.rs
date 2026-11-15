//! The light baked into vertices: storage, the sky term, and the trees' fit.

use glam::Vec3;
use xrf_geometry::{BvhTracer, Triangle};
use xrf_math::{Aabb, Vector2d, Vector3d};

use crate::geometry::mesh::geometry_face::GeometryFace;
use crate::geometry::mesh::geometry_mesh::GeometryMesh;
use crate::light::point::light_color::LightColor;
use crate::light::source::light_source::LightSource;
use crate::light::vertex::light_hemisphere::LightHemisphere;
use crate::light::vertex::light_models::LightModels;
use crate::shadow::shadow_materials::ShadowMaterials;

/// A sky light from straight above.
fn sky(energy: f32) -> LightSource {
  LightSource {
    is_directional: true,
    is_point: false,
    diffuse: Vec3::ONE,
    position: Vec3::ZERO,
    direction: Vec3::NEG_Y,
    range: 0.0,
    range_squared: 0.0,
    attenuation: [1.0, 0.0, 0.0],
    falloff: 0.0,
    energy,
  }
}

/// A flat square facing up, two faces over four corners.
fn square() -> GeometryMesh {
  let face = |vertices: [u32; 3]| GeometryFace {
    vertices,
    texture_coordinates: [Vector2d::new(0.0, 0.0); 3],
    material: 0,
    game_material: 0,
    smoothing: 1,
    source: 0,
  };

  GeometryMesh {
    vertices: vec![
      Vector3d::new(0.0, 0.0, 0.0),
      Vector3d::new(0.0, 0.0, 1.0),
      Vector3d::new(1.0, 0.0, 1.0),
      Vector3d::new(1.0, 0.0, 0.0),
    ],
    faces: vec![face([0, 1, 2]), face([0, 2, 3])],
    bounds: Aabb::from_points([Vector3d::new(0.0, 0.0, 0.0), Vector3d::new(1.0, 0.0, 1.0)]).expect("a box"),
  }
}

#[test]
fn stores_a_term_as_base_color_and_packs_it_by_flooring() {
  // A 32767th of 16, floored: 0.5 is kept a little under itself.
  assert_eq!(LightColor::get_fixed(0.5), 1023.0 / 32767.0 * 16.0);
  assert!(LightColor::get_fixed(0.3) <= 0.3 && 0.3 - LightColor::get_fixed(0.3) < 16.0 / 32767.0);
  assert_eq!(LightColor::get_byte(0.5), 127, "u8_clr floors");
  assert_eq!(LightColor::get_byte(1.5), 255);
  assert_eq!(LightColor::get_byte(-0.25), 0);
}

#[test]
fn an_open_vertex_takes_half_the_sky_and_keeps_it_through_smoothing() {
  let open: BvhTracer = BvhTracer::of_triangles(vec![Triangle::new([
    Vec3::new(100.0, -1.0, 100.0),
    Vec3::new(101.0, -1.0, 100.0),
    Vec3::new(100.0, -1.0, 101.0),
  ])]);
  let terms: Vec<f32> = LightHemisphere::of(
    &square(),
    &[sky(0.25), sky(0.5)],
    &open,
    &ShadowMaterials::new_opaque(1),
  );

  assert_eq!(terms, vec![LightColor::get_fixed(0.375); 4]);
}

#[test]
fn a_roofed_vertex_takes_no_sky() {
  let roof: BvhTracer = BvhTracer::of_triangles(vec![
    Triangle::new([
      Vec3::new(-5.0, 2.0, -5.0),
      Vec3::new(5.0, 2.0, -5.0),
      Vec3::new(5.0, 2.0, 5.0),
    ]),
    Triangle::new([
      Vec3::new(-5.0, 2.0, -5.0),
      Vec3::new(5.0, 2.0, 5.0),
      Vec3::new(-5.0, 2.0, 5.0),
    ]),
  ]);

  assert_eq!(
    LightHemisphere::of(&square(), &[sky(1.0)], &roof, &ShadowMaterials::new_opaque(2)),
    vec![0.0; 4]
  );
}

#[test]
fn a_trees_fit_is_the_least_squares_line_then_its_bias_moves_toward_the_mean() {
  let sky_terms: Vec<f64> = vec![0.0, 0.25, 0.5, 1.0];
  let placement: Vec<f64> = sky_terms.iter().map(|sky| 0.5 * sky + 0.125).collect();

  assert_eq!(LightModels::regress(&sky_terms, &placement), (0.5, 0.125));

  // On the model's sun, all 0: the scale stays, the bias steps a seventh toward the mean while the error falls.
  let (scale, bias) = LightModels::optimize(&[0.0; 4], &placement, 0.5, 0.125);
  let mean: f64 = placement.iter().sum::<f64>() / 4.0;

  assert_eq!(scale, 0.5);
  assert!(f64::from(bias) > 0.125 && f64::from(bias) <= mean);
}
