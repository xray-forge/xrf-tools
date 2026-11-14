/// How a [`crate::light::light_plan::LightPlan`] describes the rays toward lights at positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LightLampRays {
  /// Each ray made on the CPU, as `LightPoint` makes it: the light the backends give alike to the last bit, for a
  /// lighter whose rays are few.
  Single,
  /// A light's rays as one packet, each made where it is traced from its point and the light's position: the GPU's
  /// arithmetic may part from the CPU's in a last bit, but nothing per ray is stored or uploaded, for a lighter whose
  /// rays run to billions.
  Packet,
}
