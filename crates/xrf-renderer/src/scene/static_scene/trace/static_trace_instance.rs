use xrf_math::glam::{Mat3, Vec3};
use xrf_renderer_core::ShaderStruct;

/// A top-level acceleration structure instance as a traced hit reads it by custom index: its first geometry record,
/// its place (which also gives its hemisphere), and its transform's effect on cut-out densities
/// (`StaticTraceDensity`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceInstance")]
pub(crate) struct StaticTraceInstance {
  pub first_record: u32,
  pub place: u32,
  /// Log2 added to each triangle's density under uniform scale, minus log2 of the scale; zero under uneven scale.
  pub density_scale: f32,
  /// `UNEVEN` under uneven scale, where the density depends on the triangle's facing and is worked out on read.
  pub flags: u32,
}

impl StaticTraceInstance {
  pub(crate) const UNEVEN: u32 = 1;

  /// Relative tolerance on axis length and orthogonality for a transform to count as uniformly scaled.
  const EVEN_TOLERANCE: f32 = 1e-4;

  /// An instance from a row-major 3x4 `transform` (three rows of the linear part, each ending in the translation).
  pub(crate) fn new(first_record: u32, place: u32, transform: &[f32; 12]) -> Self {
    let linear: Mat3 = Mat3::from_cols(
      Vec3::new(transform[0], transform[4], transform[8]),
      Vec3::new(transform[1], transform[5], transform[9]),
      Vec3::new(transform[2], transform[6], transform[10]),
    );

    match Self::get_even_scale(&linear) {
      Some(scale) => Self {
        first_record,
        place,
        density_scale: -scale.log2(),
        flags: 0,
      },
      None => Self {
        first_record,
        place,
        density_scale: 0.0,
        flags: Self::UNEVEN,
      },
    }
  }

  /// The uniform scale of a rotation-and-scale linear part (equal, orthogonal axes); none otherwise.
  fn get_even_scale(linear: &Mat3) -> Option<f32> {
    let axes: [Vec3; 3] = [linear.x_axis, linear.y_axis, linear.z_axis];
    let lengths: [f32; 3] = axes.map(Vec3::length);
    let scale: f32 = (lengths[0] + lengths[1] + lengths[2]) / 3.0;
    let tolerance: f32 = Self::EVEN_TOLERANCE * scale;
    let is_square: bool = [(0, 1), (1, 2), (2, 0)]
      .iter()
      .all(|(a, b)| axes[*a].dot(axes[*b]).abs() <= tolerance * scale);

    (scale > 0.0 && lengths.iter().all(|length| (length - scale).abs() <= tolerance) && is_square).then_some(scale)
  }
}
