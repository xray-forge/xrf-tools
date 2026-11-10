use glam::{Mat3, Vec3};
use xrf_renderer_core::ShaderStruct;

/// One top-level instance of the level's acceleration structure as a traced hit reads it, by the instance's custom
/// index: the first record of its geometries, the place it stands at, whose hemisphere lights it, and what its
/// transform does to its cut-out triangles' densities (`StaticTraceDensity`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceInstance")]
pub(crate) struct StaticTraceInstance {
  pub first_record: u32,
  pub place: u32,
  /// The base-2 logarithm its transform adds to each of its triangles' densities where it turns and scales alike each
  /// way: minus that of its scale. Nought where it scales unevenly.
  pub density_scale: f32,
  /// `UNEVEN` where its transform scales unevenly, so each triangle's density changes with the way it faces, worked
  /// out where it is read.
  pub flags: u32,
}

impl StaticTraceInstance {
  pub(crate) const UNEVEN: u32 = 1;

  /// How far a transform's axes may differ in length, or lean from square, as a share of their length, and still be
  /// taken as scaling alike each way.
  const EVEN_TOLERANCE: f32 = 1e-4;

  /// An instance standing its first record's geometries at a place by a transform, three rows of its linear part each
  /// closed by its translation's coordinate.
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

  /// The one scale a linear part stands everything at, where it is a turn by it: its axes as long and square to each
  /// other; none otherwise.
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
