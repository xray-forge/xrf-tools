use xrf_math::Vector3d;

use crate::geometry::mesh::geometry_mesh::GeometryMesh;

/// A box as xrLC's split phases reckon it (`Fbox`), each operation in the order of its single-precision steps, the
/// axes indexable so a cut can move along one.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GeometrySplitBox {
  pub(crate) min: [f32; 3],
  pub(crate) max: [f32; 3],
}

impl GeometrySplitBox {
  /// The box of `faces`' corners (`Fbox::modify` from `invalidate`).
  pub(crate) fn of(mesh: &GeometryMesh, faces: &[u32]) -> Self {
    let mut bounds: Self = Self {
      min: [f32::MAX; 3],
      max: [f32::MIN; 3],
    };

    for face in faces {
      for corner in mesh.faces[*face as usize].get_corners(&mesh.vertices) {
        for (axis, value) in corner.to_array().into_iter().enumerate() {
          bounds.min[axis] = bounds.min[axis].min(value);
          bounds.max[axis] = bounds.max[axis].max(value);
        }
      }
    }

    bounds
  }

  /// `getsize`.
  pub(crate) fn get_size(&self) -> [f32; 3] {
    std::array::from_fn(|axis| self.max[axis] - self.min[axis])
  }

  /// `getvolume`.
  pub(crate) fn get_volume(&self) -> f32 {
    let [x, y, z] = self.get_size();

    x * y * z
  }

  /// `grow`.
  pub(crate) fn grow(&self, by: f32) -> Self {
    Self {
      min: self.min.map(|value| value - by),
      max: self.max.map(|value| value + by),
    }
  }

  /// `merge`: the box around both.
  pub(crate) fn merge(&self, other: &Self) -> Self {
    Self {
      min: std::array::from_fn(|axis| self.min[axis].min(other.min[axis])),
      max: std::array::from_fn(|axis| self.max[axis].max(other.max[axis])),
    }
  }

  /// `contains`: `point` within the closed box.
  pub(crate) fn contains(&self, point: &Vector3d) -> bool {
    point
      .to_array()
      .into_iter()
      .enumerate()
      .all(|(axis, value)| value >= self.min[axis] && value <= self.max[axis])
  }

  /// `setup_bbs`: the box halved on `axis`, its lower and upper halves.
  pub(crate) fn halve(&self, axis: usize) -> [Self; 2] {
    let half: f32 = self.get_size()[axis] / 2.0;
    let (mut lower, mut upper): (Self, Self) = (*self, *self);

    lower.max[axis] -= half;
    upper.min[axis] += half;

    [lower, upper]
  }

  /// `MakeCube`: the cube about the box's centre reaching its widest half-side (`get_CD`, then `grow` from the centre).
  pub(crate) fn get_cube(&self) -> Self {
    let half: [f32; 3] = std::array::from_fn(|axis| (self.max[axis] - self.min[axis]) * 0.5);
    let center: [f32; 3] = std::array::from_fn(|axis| self.min[axis] + half[axis]);
    let reach: f32 = half[0].max(half[1]).max(half[2]);

    Self {
      min: center.map(|value| value - reach),
      max: center.map(|value| value + reach),
    }
  }
}
