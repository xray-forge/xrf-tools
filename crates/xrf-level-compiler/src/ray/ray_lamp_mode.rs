/// How a plan of rays describes the rays toward lights at positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RayLampMode {
  /// Each ray made on the CPU, as `LightPoint` makes it, so both backends light alike to the last bit; for a lighter
  /// whose rays are few.
  Single,
  /// A light's rays as one packet, each made where it is traced. The GPU's arithmetic may part from the CPU's in a
  /// last bit, but nothing per ray is stored or uploaded; for a lighter whose rays run to billions.
  Packet,
}
