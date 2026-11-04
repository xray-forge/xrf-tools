use std::sync::Arc;
use std::time::Duration;

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::scene::scene_acoustics_state::SceneAcousticsState;
use crate::acoustics::scene::scene_geometry_state::SceneGeometryState;
use crate::acoustics::scene::simulated_source::SimulatedSource;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;
#[cfg(feature = "steam-audio")]
use crate::acoustics::steam::scene_simulation::SceneSimulation;
use crate::host::audio_scene_geometry::AudioSceneGeometry;
use crate::scene::audio_acoustics_report::AudioAcousticsReport;
use crate::scene::audio_listener::AudioListener;
use crate::voice::voice_spatial::VoiceSpatial;

/// A scene's acoustics on the control thread: where the acoustics run and the host gave the scene's geometry, a
/// simulation on a thread of its own hears each placed sound's way through the walls and the listener's reverb; the
/// control thread hands it the sounds each tick and reads what it last heard, never waiting on it.
pub(crate) struct SceneAcoustics {
  #[cfg(feature = "steam-audio")]
  acoustics: AudioAcoustics,
  /// Why the scene plays dry; none while the acoustics run.
  failure: Option<String>,
  #[cfg(feature = "steam-audio")]
  simulation: Option<SceneSimulation>,
  latest: SceneAcousticsState,
}

impl SceneAcoustics {
  pub(crate) fn open(acoustics: &AudioAcoustics, geometry: Option<Arc<dyn AudioSceneGeometry>>) -> Self {
    let failure: Option<String> = acoustics.get_failure().map(str::to_owned);

    #[cfg(feature = "steam-audio")]
    {
      let simulation: Option<SceneSimulation> = geometry
        .filter(|_| failure.is_none())
        .map(|geometry| SceneSimulation::start(acoustics.clone(), geometry));
      let geometry: SceneGeometryState = if simulation.is_some() {
        SceneGeometryState::Building
      } else {
        SceneGeometryState::Absent
      };

      Self {
        acoustics: acoustics.clone(),
        failure,
        simulation,
        latest: SceneAcousticsState::new(geometry),
      }
    }

    #[cfg(not(feature = "steam-audio"))]
    {
      let _ = geometry;

      Self {
        failure,
        latest: SceneAcousticsState::new(SceneGeometryState::Absent),
      }
    }
  }

  /// Whether placed sounds are heard through the acoustics.
  pub(crate) fn is_running(&self) -> bool {
    self.failure.is_none()
  }

  /// A placed voice's effects at the mixer's `rate`, none where they cannot be made, when the voice plays dry.
  pub(crate) fn create_voice_spatial(&self, rate: u32) -> Option<Box<VoiceSpatial>> {
    #[cfg(feature = "steam-audio")]
    {
      self
        .acoustics
        .get_steam()?
        .create_voice_effects(rate)
        .inspect_err(|error| log::warn!("A sound plays dry: {error}"))
        .ok()
        .map(Box::new)
    }

    #[cfg(not(feature = "steam-audio"))]
    {
      let _ = rate;

      None
    }
  }

  /// What the air lets through in each band between `listener` and a sound at `position`.
  pub(crate) fn get_air_absorption(&self, listener: &AudioListener, position: [f32; 3]) -> [f32; 3] {
    #[cfg(feature = "steam-audio")]
    if let Some(steam) = self.acoustics.get_steam() {
      return steam.get_air_absorption(position, listener.position);
    }

    let _ = (listener, position);

    [1.0; 3]
  }

  /// Hands the simulation where the listener and `sources` are now, and takes what it last heard.
  pub(crate) fn simulate(&mut self, listener: &AudioListener, sources: &[SimulatedSource]) {
    #[cfg(feature = "steam-audio")]
    if let Some(simulation) = &self.simulation {
      simulation.submit(listener, sources);

      if let Some(state) = simulation.take() {
        self.latest = state;
      }
    }

    #[cfg(not(feature = "steam-audio"))]
    let _ = (listener, sources);
  }

  /// How the walls shape the source `id`'s way to the listener, unobstructed until the simulation has heard it.
  pub(crate) fn get_source(&self, id: u64) -> SourceAcoustics {
    self.latest.sources.get(&id).copied().unwrap_or_default()
  }

  pub(crate) fn get_reverb_times(&self) -> [f32; 3] {
    self.latest.reverb_times
  }

  pub(crate) fn report(&self) -> AudioAcousticsReport {
    let mut report: AudioAcousticsReport = AudioAcousticsReport {
      failure: self.failure.clone(),
      reverb_times: self.latest.reverb_times,
      direct_ms: Self::to_ms(self.latest.direct_time),
      reflections_ms: Self::to_ms(self.latest.reflections_time),
      ..AudioAcousticsReport::default()
    };
    let is_running: bool = self.failure.is_none();

    match &self.latest.geometry {
      SceneGeometryState::Absent if is_running => {
        report.geometry_failure = Some(String::from("The scene has no geometry"));
      }
      SceneGeometryState::Absent | SceneGeometryState::Building => {}
      SceneGeometryState::Ready { triangles, build_time } => {
        report.triangles = Some(*triangles);
        report.build_ms = Self::to_ms(*build_time);
      }
      SceneGeometryState::Failed(reason) => report.geometry_failure = Some(reason.clone()),
    }

    report
  }

  fn to_ms(time: Duration) -> f32 {
    time.as_secs_f32() * 1000.0
  }
}
