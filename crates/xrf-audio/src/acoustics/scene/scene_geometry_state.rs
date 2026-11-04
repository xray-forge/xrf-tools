use std::time::Duration;

/// Where a scene's geometry stands in the simulation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SceneGeometryState {
  /// The host gave none: nothing occludes, and there is no reverb.
  Absent,
  /// Being read and built on the simulation's thread.
  Building,
  Ready {
    triangles: u32,
    build_time: Duration,
  },
  Failed(String),
}
