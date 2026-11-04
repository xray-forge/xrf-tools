use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::scene::scene_acoustics_state::SceneAcousticsState;
use crate::acoustics::scene::scene_geometry_state::SceneGeometryState;
use crate::acoustics::scene::simulated_source::SimulatedSource;
use crate::acoustics::scene::simulation_pace::SimulationPace;
use crate::acoustics::steam::simulated_world::SimulatedWorld;
use crate::host::audio_scene_geometry::AudioSceneGeometry;
use crate::scene::audio_listener::AudioListener;

/// One scene's simulation: it reads the scene's geometry and builds Steam Audio's scene from it, then, each time the
/// control side hands it where the listener and the sounds are, hears each sound's way through the walls (occlusion,
/// and what the walls let through) and, less often, the listener's reverb from the rays it sends around.
pub(crate) struct SceneSimulation {
  run: SimulationRun,
}

enum SimulationRun {
  RealTime(Arc<SimulationShared>),
  Stepped(Box<SteppedSimulation>),
}

struct SimulationShared {
  state: Mutex<SharedState>,
  wake: Condvar,
}

struct SharedState {
  input: Option<SimulationInput>,
  output: Option<SceneAcousticsState>,
  is_closed: bool,
}

struct SimulationInput {
  listener: AudioListener,
  sources: Vec<SimulatedSource>,
}

/// A simulation run inline: its world, none where it could not be built, what it heard last, and when on the scene's
/// clock each pass last ran.
struct SteppedSimulation {
  world: Option<SimulatedWorld>,
  last: SceneAcousticsState,
  directed_at: Option<Duration>,
  reflected_at: Option<Duration>,
  output: Option<SceneAcousticsState>,
}

impl SceneSimulation {
  /// The longest a sound's way through the walls waits to be heard again.
  const DIRECT_INTERVAL: Duration = Duration::from_millis(33);

  /// How often the listener's reverb is heard again.
  const REFLECTIONS_INTERVAL: Duration = Duration::from_millis(250);

  /// Threads the reverb's rays are traced on in real time, beside the control thread and the mixer's.
  const REVERB_THREADS: u32 = 2;

  /// Threads the reverb's rays are traced on stepped, which waits for them. With two, the reverb heard differs between
  /// any two runs of the same frames; with one it often repeats, and otherwise differs only by the rays Steam Audio
  /// draws at random.
  const STEPPED_REVERB_THREADS: u32 = 1;

  /// A simulation of the scene `geometry` holds, keeping up as `pace` says.
  pub(crate) fn start(acoustics: AudioAcoustics, geometry: Arc<dyn AudioSceneGeometry>, pace: SimulationPace) -> Self {
    let run: SimulationRun = match pace {
      SimulationPace::RealTime => SimulationRun::RealTime(Self::spawn(acoustics, geometry)),
      SimulationPace::Stepped => {
        SimulationRun::Stepped(Box::new(SteppedSimulation::build(&acoustics, geometry.as_ref())))
      }
    };

    Self { run }
  }

  /// Hands the simulation where the listener and `sources` are at `now` on the scene's clock: in real time in place of
  /// what it has not taken yet; stepped, each pass runs where its interval has passed since it last ran.
  pub(crate) fn submit(&mut self, listener: &AudioListener, sources: &[SimulatedSource], now: Duration) {
    match &mut self.run {
      SimulationRun::RealTime(shared) => shared.submit(listener, sources),
      SimulationRun::Stepped(stepped) => stepped.step(listener, sources, now),
    }
  }

  /// What the simulation heard since it was last taken, none where it has heard nothing new.
  pub(crate) fn take(&mut self) -> Option<SceneAcousticsState> {
    match &mut self.run {
      SimulationRun::RealTime(shared) => shared.lock().output.take(),
      SimulationRun::Stepped(stepped) => stepped.output.take(),
    }
  }

  fn spawn(acoustics: AudioAcoustics, geometry: Arc<dyn AudioSceneGeometry>) -> Arc<SimulationShared> {
    let shared: Arc<SimulationShared> = Arc::new(SimulationShared {
      state: Mutex::new(SharedState {
        input: None,
        output: None,
        is_closed: false,
      }),
      wake: Condvar::new(),
    });
    let running: Arc<SimulationShared> = Arc::clone(&shared);

    if let Err(error) = thread::Builder::new()
      .name(String::from("xrf-audio-acoustics"))
      .spawn(move || Self::run(&acoustics, geometry.as_ref(), &running))
    {
      shared.publish(SceneAcousticsState::new(SceneGeometryState::Failed(format!(
        "The acoustics' simulation cannot start: {error}"
      ))));
    }

    shared
  }

  fn run(acoustics: &AudioAcoustics, geometry: &dyn AudioSceneGeometry, shared: &SimulationShared) {
    let Some((world, geometry)) = Self::build(acoustics, geometry, Self::REVERB_THREADS) else {
      return;
    };
    let mut last: SceneAcousticsState = SceneAcousticsState::new(geometry);

    shared.publish(last.clone());

    let Some(mut world) = world else {
      return;
    };
    let mut reflected_at: Option<Instant> = None;

    while let Some(input) = shared.wait() {
      let run_started: Instant = Instant::now();

      last.sources = world.run_direct(&input.listener, &input.sources);
      last.direct_time = run_started.elapsed();

      if reflected_at.is_none_or(|at| at.elapsed() >= Self::REFLECTIONS_INTERVAL) {
        let reflections_started: Instant = Instant::now();

        last.reverb_times = world.run_reflections(&input.listener).unwrap_or([0.0; 3]);
        last.reflections_time = reflections_started.elapsed();
        reflected_at = Some(reflections_started);
      }

      shared.publish(last.clone());

      if let Some(rest) = Self::DIRECT_INTERVAL.checked_sub(run_started.elapsed()) {
        thread::sleep(rest);
      }
    }
  }

  /// Steam Audio's world of the scene's geometry, its reverb on `reverb_threads`, and the geometry's state: ready, or
  /// failed with no world. None where the acoustics do not run.
  fn build(
    acoustics: &AudioAcoustics,
    geometry: &dyn AudioSceneGeometry,
    reverb_threads: u32,
  ) -> Option<(Option<SimulatedWorld>, SceneGeometryState)> {
    let steam = acoustics.get_steam()?;
    let started: Instant = Instant::now();
    let built: Result<SimulatedWorld, String> = geometry
      .read_mesh()
      .map_err(|error| format!("The scene's geometry cannot be read: {error}"))
      .and_then(|mesh| SimulatedWorld::build(steam, &mesh, reverb_threads));

    Some(match built {
      Ok(world) => {
        log::info!(
          "The acoustics' scene holds {} triangles, built in {:?}",
          world.get_triangles(),
          started.elapsed()
        );

        let geometry: SceneGeometryState = SceneGeometryState::Ready {
          triangles: world.get_triangles(),
          build_time: started.elapsed(),
        };

        (Some(world), geometry)
      }
      Err(reason) => {
        log::warn!("The scene's walls shape no sound: {reason}");

        (None, SceneGeometryState::Failed(reason))
      }
    })
  }
}

impl Drop for SceneSimulation {
  fn drop(&mut self) {
    if let SimulationRun::RealTime(shared) = &self.run {
      shared.lock().is_closed = true;
      shared.wake.notify_one();
    }
  }
}

impl SimulationShared {
  fn lock(&self) -> MutexGuard<'_, SharedState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn submit(&self, listener: &AudioListener, sources: &[SimulatedSource]) {
    let mut state = self.lock();

    match &mut state.input {
      Some(input) => {
        input.listener = *listener;
        input.sources.clear();
        input.sources.extend_from_slice(sources);
      }
      None => {
        state.input = Some(SimulationInput {
          listener: *listener,
          sources: sources.to_vec(),
        });
      }
    }

    self.wake.notify_one();
  }

  /// The next hand-over, none once the scene closed.
  fn wait(&self) -> Option<SimulationInput> {
    let mut state = self.lock();

    loop {
      if state.is_closed {
        return None;
      }

      if let Some(input) = state.input.take() {
        return Some(input);
      }

      state = self.wake.wait(state).unwrap_or_else(PoisonError::into_inner);
    }
  }

  fn publish(&self, output: SceneAcousticsState) {
    self.lock().output = Some(output);
  }
}

impl SteppedSimulation {
  /// Builds the scene's world inline, telling what it built at once.
  fn build(acoustics: &AudioAcoustics, geometry: &dyn AudioSceneGeometry) -> Self {
    let (world, geometry): (Option<SimulatedWorld>, SceneGeometryState) =
      SceneSimulation::build(acoustics, geometry, SceneSimulation::STEPPED_REVERB_THREADS)
        .unwrap_or_else(|| (None, SceneGeometryState::Absent));
    let last: SceneAcousticsState = SceneAcousticsState::new(geometry);

    Self {
      world,
      output: Some(last.clone()),
      last,
      directed_at: None,
      reflected_at: None,
    }
  }

  /// Runs each pass due at `now`: a source's way through the walls every [`SceneSimulation::DIRECT_INTERVAL`], the
  /// listener's reverb every [`SceneSimulation::REFLECTIONS_INTERVAL`], both at once the first time.
  fn step(&mut self, listener: &AudioListener, sources: &[SimulatedSource], now: Duration) {
    let Some(world) = &mut self.world else {
      return;
    };
    let is_due = |at: Option<Duration>, interval: Duration| at.is_none_or(|at| now < at || now - at >= interval);

    if !is_due(self.directed_at, SceneSimulation::DIRECT_INTERVAL) {
      return;
    }

    let run_started: Instant = Instant::now();

    self.last.sources = world.run_direct(listener, sources);
    self.last.direct_time = run_started.elapsed();
    self.directed_at = Some(now);

    if is_due(self.reflected_at, SceneSimulation::REFLECTIONS_INTERVAL) {
      let reflections_started: Instant = Instant::now();

      self.last.reverb_times = world.run_reflections(listener).unwrap_or([0.0; 3]);
      self.last.reflections_time = reflections_started.elapsed();
      self.reflected_at = Some(now);
    }

    self.output = Some(self.last.clone());
  }
}
