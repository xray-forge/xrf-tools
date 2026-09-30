use serde::{Deserialize, Serialize};

/// How much sky and light reach a dynamic object from each way along each axis, `CROS_impl::hemi_cube`: the faces
/// toward `+x +y +z`, then toward `-x -y -z`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HemiCube {
  pub faces: [f32; 6],
}

impl HemiCube {
  /// Faces a cube has.
  pub const FACES: usize = 6;

  /// `accum_hemi`: a direction's share added to the face it points toward along each axis.
  pub fn accumulate(&mut self, direction: &[f32; 3], scale: f32) {
    for (axis, component) in direction.iter().enumerate() {
      if *component > 0.0 {
        self.faces[axis] += component * scale;
      } else {
        self.faces[axis + 3] -= component * scale;
      }
    }
  }

  /// The face opposite each, `(i + NUM_FACES / 2) % NUM_FACES`.
  pub const fn opposite(face: usize) -> usize {
    (face + Self::FACES / 2) % Self::FACES
  }

  /// The same cube in renderer space, whose `z` runs the other way: the faces toward `+z` and `-z` swap.
  pub fn to_renderer_space(&self) -> Self {
    let [px, py, pz, nx, ny, nz] = self.faces;

    Self {
      faces: [px, py, nz, nx, ny, pz],
    }
  }
}
