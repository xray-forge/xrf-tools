use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_level::{LevelCformFace, LevelCformGeometry, LevelCformTracer, LevelLight};
use xrf_math::Vector3d;

use crate::data::lights::hemi_cube::HemiCube;
use crate::pack::lights::hemi_estimator::HemiEstimator;

/// `+y` and `-y`, among the cube's faces.
const UP: usize = 1;
const DOWN: usize = 4;

fn face(vertices: [u32; 3]) -> LevelCformFace {
  LevelCformFace {
    is_shadow_suppressed: false,
    is_wallmark_suppressed: false,
    material: 0,
    sector: 0,
    vertices,
  }
}

/// A roof a hundred metres across, three metres up.
fn roof() -> XrfResult<LevelCformTracer> {
  let geometry: LevelCformGeometry = LevelCformGeometry::new(
    vec![
      Vector3d::new(-50.0, 3.0, -50.0),
      Vector3d::new(50.0, 3.0, -50.0),
      Vector3d::new(50.0, 3.0, 50.0),
      Vector3d::new(-50.0, 3.0, 50.0),
    ],
    vec![face([0, 2, 1]), face([0, 3, 2])],
  )?;

  Ok(LevelCformTracer::new(&geometry))
}

fn open() -> XrfResult<LevelCformTracer> {
  Ok(LevelCformTracer::new(&LevelCformGeometry::new(Vec::new(), Vec::new())?))
}

fn point_light(position: Vector3d<f32>) -> LevelLight {
  LevelLight {
    attenuation_constant: 1.0,
    attenuation_linear: 0.0,
    attenuation_quadratic: 0.0,
    diffuse: Vector3d::new(1.0, 1.0, 1.0),
    direction: Vector3d::new(0.0, -1.0, 0.0),
    energy: 1.0,
    falloff: 0.0,
    kind: LevelLight::KIND_POINT,
    level: 0,
    position,
    range: 10.0,
    range_squared: 100.0,
    triangle: [0; 3].map(|_| Vector3d::new(0.0, 0.0, 0.0)),
  }
}

// Under open sky every sample passes: the upward face takes the upward components of the 26 directions times 0.08,
// and nothing lights the face toward the ground but the least a face holds.
#[test]
fn lights_an_object_under_open_sky_from_above() -> XrfResult {
  let cube: HemiCube = HemiEstimator::new(Arc::new(open()?), &[])
    .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
    .cube;

  assert!((cube.faces[UP] - 0.8094).abs() < 1e-3, "{:?}", cube.faces);
  assert!((cube.faces[DOWN] - 1.0 / 255.0).abs() < 1e-6);

  Ok(())
}

// `hemi_value`: the share of the 26 samples open, times 0.08. Under the roof only the ten level samples pass.
#[test]
fn shares_the_open_sky_as_the_engine_counts_it() -> XrfResult {
  let open: f32 = HemiEstimator::new(Arc::new(open()?), &[])
    .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
    .sky;
  let roofed: f32 = HemiEstimator::new(Arc::new(roof()?), &[])
    .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
    .sky;

  assert!((open - 0.08).abs() < 1e-6, "{open}");
  assert!((roofed - 10.0 / 26.0 * 0.08).abs() < 1e-6, "{roofed}");

  Ok(())
}

#[test]
fn darkens_an_object_under_a_roof_from_above_and_keeps_its_sides_open() -> XrfResult {
  let cube: HemiCube = HemiEstimator::new(Arc::new(roof()?), &[])
    .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
    .cube;

  // The roof reaches past fifty metres along every rising sample; only the level ones reach out under it.
  assert!((cube.faces[UP] - 1.0 / 255.0).abs() < 1e-6, "{:?}", cube.faces);
  assert!(cube.faces[0] > 0.1);

  Ok(())
}

// A light a metre and a half to the side, unoccluded, adds toward it, and a tenth of that to the opposite face.
#[test]
fn adds_a_compiled_light_that_reaches_the_object_toward_the_face_it_comes_from() -> XrfResult {
  let lights: Vec<LevelLight> = vec![point_light(Vector3d::new(1.5, 1.15, 0.0))];
  let cube: HemiCube = HemiEstimator::new(Arc::new(roof()?), &lights)
    .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
    .cube;
  let unlit: HemiCube = HemiEstimator::new(Arc::new(roof()?), &[])
    .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
    .cube;

  // Its colour's mean, halved as a seen light settles, times its attenuation of one and the light scale.
  assert!(
    (cube.faces[0] - unlit.faces[0] - 0.1 * 0.9).abs() < 1e-4,
    "{:?}",
    cube.faces
  );
  assert!(
    (cube.faces[3] - unlit.faces[3] - 0.1 * 0.1).abs() < 1e-4,
    "{:?}",
    cube.faces
  );

  Ok(())
}

#[test]
fn ignores_a_light_the_form_hides_and_one_out_of_its_reach() -> XrfResult {
  let hidden: Vec<LevelLight> = vec![point_light(Vector3d::new(0.0, 5.0, 0.0))];
  let far: Vec<LevelLight> = vec![point_light(Vector3d::new(20.0, 1.15, 0.0))];
  let unlit: HemiCube = HemiEstimator::new(Arc::new(roof()?), &[])
    .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
    .cube;

  assert_eq!(
    HemiEstimator::new(Arc::new(roof()?), &hidden)
      .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
      .cube,
    unlit
  );
  assert_eq!(
    HemiEstimator::new(Arc::new(roof()?), &far)
      .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
      .cube,
    unlit
  );

  Ok(())
}

// A light written with no attenuation would add without end, and would draw its object white wherever it stood.
#[test]
fn adds_nothing_of_a_light_without_attenuation() -> XrfResult {
  let unattenuated: Vec<LevelLight> = vec![LevelLight {
    attenuation_constant: 0.0,
    ..point_light(Vector3d::new(1.5, 1.15, 0.0))
  }];
  let unlit: HemiCube = HemiEstimator::new(Arc::new(roof()?), &[])
    .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
    .cube;

  assert_eq!(
    HemiEstimator::new(Arc::new(roof()?), &unattenuated)
      .estimate(&Vector3d::new(0.0, 1.0, 0.0), 0.5)
      .cube,
    unlit
  );

  Ok(())
}

#[test]
fn swaps_the_faces_along_z_into_renderer_space() {
  let cube: HemiCube = HemiCube {
    faces: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
  };

  assert_eq!(cube.to_renderer_space().faces, [1.0, 2.0, 6.0, 4.0, 5.0, 3.0]);
}
