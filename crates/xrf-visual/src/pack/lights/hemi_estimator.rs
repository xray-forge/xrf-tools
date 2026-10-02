use xrf_level::{LevelCformTracer, LevelLight};
use xrf_math::Vector3d;

use crate::data::lights::hemi_cube::HemiCube;

/// `lt_hemisamples`: the directions a dynamic object's sky is sampled along.
const SKY_SAMPLES: usize = 26;

/// `hdir`, the directions sampled (`Layers/xrRender/LightTrack.cpp`).
const SKY_DIRECTIONS: [[f32; 3]; SKY_SAMPLES] = [
  [-0.26287, 0.52573, 0.80902],
  [0.27639, 0.44721, 0.85065],
  [-0.95106, 0.00000, 0.30902],
  [-0.95106, 0.00000, -0.30902],
  [0.58779, 0.00000, -0.80902],
  [0.58779, 0.00000, 0.80902],
  [-0.00000, 0.00000, 1.00000],
  [0.52573, 0.85065, 0.00000],
  [-0.26287, 0.52573, -0.80902],
  [-0.42533, 0.85065, 0.30902],
  [0.95106, 0.00000, 0.30902],
  [0.95106, 0.00000, -0.30902],
  [0.00000, 1.00000, 0.00000],
  [-0.58779, 0.00000, 0.80902],
  [-0.72361, 0.44721, 0.52573],
  [-0.72361, 0.44721, -0.52573],
  [-0.58779, 0.00000, -0.80902],
  [0.16246, 0.85065, -0.50000],
  [0.89443, 0.44721, 0.00000],
  [-0.85065, 0.52573, -0.00000],
  [0.16246, 0.85065, 0.50000],
  [0.68819, 0.52573, -0.50000],
  [0.27639, 0.44721, -0.85065],
  [0.00000, 0.00000, -1.00000],
  [-0.42533, 0.85065, -0.30902],
  [0.68819, 0.52573, 0.50000],
];

/// How far a sky sample reaches before it counts as open (`calc_sky_hemi_value`).
const SKY_RANGE: f32 = 50.0;

/// `ps_r2_dhemi_sky_scale`: what one open sample adds to the faces it points toward.
const SKY_SCALE: f32 = 0.08;

/// `ps_r2_dhemi_light_scale`: what a light's reach is weighted by.
const LIGHT_SCALE: f32 = 0.2;

/// `ps_r2_dhemi_light_flow`: how much of a light's share goes to the opposite face instead.
const LIGHT_FLOW: f32 = 0.1;

/// The least a face holds (`minHemiValue`).
const MIN_FACE: f32 = 1.0 / 255.0;

/// How far above its sphere's centre an object is sampled from, of its radius (`CROS_impl::update`).
const SAMPLE_LIFT: f32 = 0.3;

/// What a light it sees settles at, `energy / 2` once its visibility energy reaches one (`prepare_lights`).
const SEEN_LIGHT_SHARE: f32 = 0.5;

/// A dynamic object's lighting as `CROS_impl` settles on it standing still: the share of the sky it sees along each
/// sampled direction, and the level's compiled lights (`build.lights`, `CLight_DB::LoadHemi`) that reach it unoccluded,
/// each added to the faces it comes from.
pub struct HemiEstimator {
  tracer: LevelCformTracer,
  lights: Vec<LevelLight>,
  /// The sampled directions, unit length, normalised once rather than for every object.
  sky: [[f32; 3]; SKY_SAMPLES],
}

impl HemiEstimator {
  /// An estimator over the collision form, lit by the level's compiled lights, of which only the point ones count.
  pub fn new(tracer: LevelCformTracer, lights: &[LevelLight]) -> Self {
    Self {
      lights: lights.iter().filter(|light| light.is_point()).cloned().collect(),
      sky: SKY_DIRECTIONS.map(|direction| normalize(&direction)),
      tracer,
    }
  }

  /// The compiled lights that light objects: the point ones.
  pub fn get_light_count(&self) -> usize {
    self.lights.len()
  }

  /// The cube of an object whose visual's sphere, where it stands, is centred at `centre`, in engine space.
  pub fn estimate(&self, centre: &Vector3d<f32>, radius: f32) -> HemiCube {
    let position: [f32; 3] = [centre.x, centre.y + SAMPLE_LIFT * radius, centre.z];
    let origin: Vector3d<f32> = Vector3d::new(position[0], position[1], position[2]);
    let mut cube: HemiCube = HemiCube::default();

    for direction in &self.sky {
      if !self.tracer.is_blocked(
        &origin,
        &Vector3d::new(direction[0], direction[1], direction[2]),
        SKY_RANGE,
      ) {
        cube.accumulate(direction, SKY_SCALE);
      }
    }

    let lit: HemiCube = self.light(&position, radius);

    for face in 0..HemiCube::FACES {
      cube.faces[face] += lit.faces[face] * (1.0 - LIGHT_FLOW) + LIGHT_FLOW * lit.faces[HemiCube::opposite(face)];
      cube.faces[face] = cube.faces[face].max(MIN_FACE);
    }

    cube
  }

  /// What the lights that reach a point add, each toward the face it comes from (`CROS_impl::update`, R2).
  fn light(&self, position: &[f32; 3], radius: f32) -> HemiCube {
    let mut cube: HemiCube = HemiCube::default();

    for light in &self.lights {
      let source: [f32; 3] = [light.position.x, light.position.y, light.position.z];
      let toward: [f32; 3] = [
        position[0] - source[0],
        position[1] - source[1],
        position[2] - source[2],
      ];
      let distance: f32 = length(&toward);

      if distance >= radius + light.range || distance <= f32::EPSILON {
        continue;
      }

      // Traced from the light to the point, as `prepare_lights` traces a light's visibility.
      let along: Vector3d<f32> = Vector3d::new(toward[0] / distance, toward[1] / distance, toward[2] / distance);

      if self
        .tracer
        .is_blocked(&Vector3d::new(source[0], source[1], source[2]), &along, distance)
      {
        continue;
      }

      let attenuation: f32 = (1.0
        / (light.attenuation_constant
          + light.attenuation_linear * distance
          + light.attenuation_quadratic * distance * distance)
        - distance * light.falloff)
        .max(0.0);

      // A light the compiler wrote with no attenuation at all reaches everything infinitely: nothing to add.
      if !attenuation.is_finite() {
        continue;
      }

      let brightness: f32 = (light.diffuse.x + light.diffuse.y + light.diffuse.z) / 3.0 * SEEN_LIGHT_SHARE;

      cube.accumulate(&[-along.x, -along.y, -along.z], brightness * attenuation * LIGHT_SCALE);
    }

    cube
  }
}

fn length(vector: &[f32; 3]) -> f32 {
  (vector[0] * vector[0] + vector[1] * vector[1] + vector[2] * vector[2]).sqrt()
}

fn normalize(vector: &[f32; 3]) -> [f32; 3] {
  let length: f32 = length(vector);

  [vector[0] / length, vector[1] / length, vector[2] / length]
}
