use std::collections::HashMap;
use std::time::Duration;

use crate::acoustics::scene::scene_geometry_state::SceneGeometryState;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;

/// What a scene's simulation last heard: each source's way to the listener, the listener's reverb, and its geometry.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneAcousticsState {
  pub(crate) sources: HashMap<u64, SourceAcoustics>,
  /// Each band's decay time around the listener, in seconds.
  pub(crate) reverb_times: [f32; 3],
  pub(crate) geometry: SceneGeometryState,
  pub(crate) direct_time: Duration,
  pub(crate) reflections_time: Duration,
}

impl SceneAcousticsState {
  /// Nothing heard yet, the geometry where `geometry` says.
  pub(crate) fn new(geometry: SceneGeometryState) -> Self {
    Self {
      sources: HashMap::new(),
      reverb_times: [0.0; 3],
      geometry,
      direct_time: Duration::ZERO,
      reflections_time: Duration::ZERO,
    }
  }
}
