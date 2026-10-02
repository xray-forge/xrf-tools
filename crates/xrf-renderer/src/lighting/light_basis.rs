use glam::{Vec3, Vec4};
use xrf_math::{EPS, Vector3d};
use xrf_visual::{LightDescription, LightKind};

/// How near parallel to the world's up a spot may point before `compute_xf_spot` takes the world's forward instead.
const PARALLEL: f32 = 0.99;

/// Where a light stands and the axes it lights along: a spot's `compute_xf_spot` basis, a point's facing down `-z`.
#[derive(Clone, Copy, Debug)]
pub struct LightBasis {
  pub position: Vec3,
  pub direction: Vec3,
  pub right: Vec3,
  pub up: Vec3,
}

impl LightBasis {
  /// The direction, and the right the lamp gives made square to it through the up they make, or the world's up where
  /// it gives none. Crossed in the renderer's mirrored space, the engine's cross products turn their sign.
  pub fn of(light: &LightDescription) -> Self {
    let position: Vec3 = to_vec3(&light.position);

    if !matches!(light.kind, LightKind::Spot) {
      return Self {
        position,
        direction: Vec3::NEG_Z,
        right: Vec3::X,
        up: Vec3::Y,
      };
    }

    let direction: Vec3 = to_vec3(&light.direction).normalize_or(Vec3::NEG_Z);
    let given: Vec3 = to_vec3(&light.right);
    let (right, up): (Vec3, Vec3) = if given.length_squared() > EPS {
      let up: Vec3 = -direction.cross(given.normalize()).normalize_or(Vec3::Y);

      (-up.cross(direction).normalize_or(Vec3::X), up)
    } else {
      // The engine's world forward, `+z`, is the renderer's `-z`.
      let world_up: Vec3 = if Vec3::Y.dot(direction).abs() > PARALLEL {
        Vec3::NEG_Z
      } else {
        Vec3::Y
      };
      let right: Vec3 = -world_up.cross(direction).normalize_or(Vec3::X);

      (right, -direction.cross(right).normalize_or(Vec3::Y))
    };

    Self {
      position,
      direction,
      right,
      up,
    }
  }

  /// The least sphere around all a light reaches, as far as its range strays, which it is culled and binned by: a
  /// point's range; a narrow spot's the one through its apex and its rim, a wide one's its rim's.
  pub fn get_bound(&self, light: &LightDescription) -> Vec4 {
    let reach: f32 = light.range + light.range_jitter;

    if !matches!(light.kind, LightKind::Spot) || light.cone >= std::f32::consts::PI {
      return self.position.extend(reach);
    }

    let half: f32 = light.cone / 2.0;
    let is_narrow: bool = half <= std::f32::consts::FRAC_PI_4;
    // The two meet at a quarter turn's cone: centre and radius both `reach / sqrt(2)`.
    let along: f32 = if is_narrow {
      reach / (2.0 * half.cos())
    } else {
      reach * half.cos()
    };
    let radius: f32 = if is_narrow { along } else { reach * half.sin() };

    (self.position + self.direction * along).extend(radius)
  }

  /// `light::spatial_move`'s sphere, which the engine fades a shadowed light and sizes its maps by: a point's range; a
  /// spot's past its cone, wider than it reaches.
  pub fn get_spatial_sphere(&self, light: &LightDescription) -> Vec4 {
    if !matches!(light.kind, LightKind::Spot) {
      return self.position.extend(light.range);
    }

    let half: f32 = light.cone / 2.0;
    let is_wide: bool = light.cone >= std::f32::consts::FRAC_PI_2;
    let radius: f32 = if is_wide {
      light.range * half.tan()
    } else {
      light.range / (2.0 * half.cos() * half.cos())
    };
    let along: f32 = if is_wide { light.range } else { radius };

    (self.position + self.direction * along).extend(radius)
  }

  /// `light::spatial_move` for an `OMNIPART`, one face of a shadowed point: its range over root two across, standing
  /// that far along the face.
  pub fn get_face_sphere(&self, light: &LightDescription, direction: Vec3) -> Vec4 {
    let radius: f32 = light.range * std::f32::consts::FRAC_1_SQRT_2;

    (self.position + direction * radius).extend(radius)
  }
}

/// `ps_r2_slight_fade`: what a shadowed light's screen area is scaled by before it fades (`xrRender_console.cpp`).
const SHADOWED_FADE: f32 = 0.5;

/// `light::get_LOD`: how far a shadowed light has faded, by its sphere's share of the screen, one whole and zero gone,
/// against the progressive meshes' own `start` and `end` thresholds.
pub fn to_light_lod(sphere: Vec4, eye: Vec3, start: f32, end: f32) -> f32 {
  // `EPS` added to the squared distance, so a camera inside the light divides by no zero.
  let area: f32 = SHADOWED_FADE * sphere.w / (eye.distance_squared(sphere.truncate()) + EPS);

  if start > end {
    ((area - end) / (start - end)).clamp(0.0, 1.0).sqrt()
  } else {
    1.0
  }
}

/// `(mean + luminance) / 2`, which `compute_xf_spot` sizes a brighter light's maps larger by.
pub fn to_light_intensity(color: Vec3) -> f32 {
  (color.element_sum() / 3.0 + color.dot(Vec3::new(0.2125, 0.7154, 0.0721))) / 2.0
}

/// A level vector as the renderer computes with it.
pub fn to_vec3(vector: &Vector3d) -> Vec3 {
  Vec3::new(vector.x, vector.y, vector.z)
}
