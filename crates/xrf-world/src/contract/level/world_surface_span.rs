use serde::{Deserialize, Serialize};

/// The range a base texture coordinate covers over some of a level's geometry.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSurfaceSpan {
  pub u_min: f32,
  pub u_max: f32,
  pub v_min: f32,
  pub v_max: f32,
}

impl WorldSurfaceSpan {
  pub fn at(u: f32, v: f32) -> Self {
    Self {
      u_min: u,
      u_max: u,
      v_min: v,
      v_max: v,
    }
  }

  /// The range covering both.
  pub fn merge(self, other: Self) -> Self {
    Self {
      u_min: self.u_min.min(other.u_min),
      u_max: self.u_max.max(other.u_max),
      v_min: self.v_min.min(other.v_min),
      v_max: self.v_max.max(other.v_max),
    }
  }

  pub fn get_area(&self) -> f32 {
    (self.u_max - self.u_min) * (self.v_max - self.v_min)
  }
}
