use xrf_level::LevelVertexQuantizer;
use xrf_math::Vector3d;

/// A face corner's tangent and binormal as xrLC keeps them after its tangent-basis pass, `base_basis`: each a byte a
/// component, `x`, `y`, `z`, from `[-1, 1]` to `[0, 255]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeometryBasis {
  pub tangent: [u8; 3],
  pub binormal: [u8; 3],
}

impl GeometryBasis {
  /// The basis a vertex has before any pass gives it one, `base_basis::set(0, 0, 0)`.
  pub const NONE: Self = Self {
    tangent: [127; 3],
    binormal: [127; 3],
  };

  /// A mender's tangent and binormal as `base_basis::set` keeps them.
  pub fn of(tangent: &Vector3d, binormal: &Vector3d) -> Self {
    Self {
      tangent: Self::quantize(tangent),
      binormal: Self::quantize(binormal),
    }
  }

  /// `base_basis::similar` on both directions: every component within one step.
  pub fn is_similar(&self, other: &Self) -> bool {
    let within = |first: [u8; 3], second: [u8; 3]| (0..3).all(|axis| first[axis].abs_diff(second[axis]) <= 1);

    within(self.tangent, other.tangent) && within(self.binormal, other.binormal)
  }

  /// `base_basis::set`: the direction normalized where it has a length, then a byte a component as `u8_vec4` takes it.
  fn quantize(direction: &Vector3d) -> [u8; 3] {
    let length_squared: f32 = direction.x * direction.x + direction.y * direction.y + direction.z * direction.z;
    let normalized: Vector3d = if length_squared > f32::MIN_POSITIVE {
      let scale: f32 = (1.0 / length_squared).sqrt();

      Vector3d::new(direction.x * scale, direction.y * scale, direction.z * scale)
    } else {
      *direction
    };
    let [z, y, x, _] = LevelVertexQuantizer::get_direction(&normalized, 0);

    [x, y, z]
  }
}
