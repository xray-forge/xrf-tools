use crate::data::lights::hemi_cube::HemiCube;

/// How a dynamic object is lit standing still, as `CROS_impl` settles on it: its hemisphere cube, and its scalar sky
/// share, `hemi_value`, which a forward-drawn model's `L_material.x` carries.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HemiEstimate {
  pub cube: HemiCube,
  /// The share of the sampled sky that is open, times `ps_r2_dhemi_sky_scale`, plus what the lights add.
  pub sky: f32,
}
