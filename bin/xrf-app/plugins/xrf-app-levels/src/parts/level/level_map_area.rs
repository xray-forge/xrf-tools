use serde::{Deserialize, Serialize};

/// The ground a level map covers, engine space: its least and greatest `x` and `z`, as `bound_rect` places it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelMapArea {
  /// The first corner, the least `x` and `z`.
  pub min_x: f32,
  pub min_z: f32,
  /// The second corner, the greatest `x` and `z`.
  pub max_x: f32,
  pub max_z: f32,
}

impl LevelMapArea {
  /// Refuses an area spanning no ground, which places no point on a map.
  ///
  /// # Errors
  ///
  /// Returns an error naming the corners when either least edge is not below its greatest.
  pub fn require_ground(&self) -> Result<(), String> {
    let Self {
      min_x,
      min_z,
      max_x,
      max_z,
    } = *self;

    if min_x < max_x && min_z < max_z {
      Ok(())
    } else {
      Err(format!(
        "A level map's area spans no ground: {min_x}, {min_z}, {max_x}, {max_z}"
      ))
    }
  }
}
