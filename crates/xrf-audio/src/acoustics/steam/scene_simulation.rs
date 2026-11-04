use std::collections::HashMap;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

use audionimbus::{
  CoordinateSystem, DefaultRayTracer, Direct, DirectSimulationParameters, DirectSimulationSettings, Material,
  Occlusion, OcclusionAlgorithm, Parametric, ParametricParameters, ParametricSettings, Point, Reflections,
  ReflectionsSharedInputs, Scene, SimulationInputs, SimulationParameters, SimulationSettings, SimulationSharedInputs,
  Simulator, Source, StaticMesh, StaticMeshSettings, Transmission, TransmissionParameters, Triangle,
};

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::scene::scene_acoustics_state::SceneAcousticsState;
use crate::acoustics::scene::scene_geometry_state::SceneGeometryState;
use crate::acoustics::scene::simulated_source::SimulatedSource;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;
use crate::acoustics::steam::mixer_reverb::MixerReverb;
use crate::acoustics::steam::steam_audio::SteamAudio;
use crate::acoustics::steam::steam_space::SteamSpace;
use crate::host::audio_scene_geometry::AudioSceneGeometry;
use crate::host::audio_scene_mesh::AudioSceneMesh;
use crate::mixer::audio_mixer::AudioMixer;
use crate::scene::audio_listener::AudioListener;

type DirectSimulator = Simulator<DefaultRayTracer, Direct>;
type DirectSource = Source<Direct>;
type ReverbSimulator = Simulator<DefaultRayTracer, (), Reflections, (), Parametric>;
type ReverbSource = Source<(), Reflections, (), Parametric>;

/// One scene's simulation on a thread of its own: it reads the scene's geometry and builds Steam Audio's scene from it,
/// then, each time the control thread hands it where the listener and the sounds are, hears each sound's way through
/// the walls (occlusion, and what the walls let through) and, less often, the listener's reverb from the rays it
/// sends around. Only the latest hand-over is simulated; what it heard waits for the control thread to take.
///
/// Dropping it lets the thread end once its current run is done, without waiting for it.
pub(crate) struct SceneSimulation {
  shared: Arc<SimulationShared>,
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

/// Steam Audio's scene and the simulators tracing it.
struct SimulatedWorld {
  _scene: Scene,
  direct: DirectSimulator,
  sources: HashMap<u64, DirectSource>,
  reverb: ReverbSimulator,
  reverb_source: ReverbSource,
  triangles: u32,
}

impl SceneSimulation {
  /// The longest a sound's way through the walls waits to be heard again.
  const DIRECT_INTERVAL: Duration = Duration::from_millis(33);

  /// How often the listener's reverb is heard again.
  const REFLECTIONS_INTERVAL: Duration = Duration::from_millis(250);

  /// Points sampled around each sound: more makes a sound slide into occlusion rather than step.
  const OCCLUSION_SAMPLES: u32 = 16;

  /// How big a sound is taken to be for its occlusion, in metres.
  const OCCLUSION_RADIUS: f32 = 1.0;

  /// Walls in the way whose transmission is counted, nearest the listener first.
  const TRANSMISSION_RAYS: u32 = 4;

  /// Rays sent from the listener, each reflected at most this many times, to hear its reverb.
  const REVERB_RAYS: u32 = 4096;
  const REVERB_BOUNCES: u32 = 16;

  pub(crate) fn start(acoustics: AudioAcoustics, geometry: Arc<dyn AudioSceneGeometry>) -> Self {
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
      shared.lock().output = Some(SceneAcousticsState::new(SceneGeometryState::Failed(format!(
        "The acoustics' simulation cannot start: {error}"
      ))));
    }

    Self { shared }
  }

  /// Hands the simulation where the listener and `sources` are now, in place of what it has not taken yet.
  pub(crate) fn submit(&self, listener: &AudioListener, sources: &[SimulatedSource]) {
    let mut state = self.shared.lock();

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

    self.shared.wake.notify_one();
  }

  /// What the simulation heard since it was last taken, none where it has heard nothing new.
  pub(crate) fn take(&self) -> Option<SceneAcousticsState> {
    self.shared.lock().output.take()
  }

  fn run(acoustics: &AudioAcoustics, geometry: &dyn AudioSceneGeometry, shared: &SimulationShared) {
    let Some(steam) = acoustics.get_steam() else {
      return;
    };
    let started: Instant = Instant::now();
    let mut world: SimulatedWorld = match geometry
      .read_mesh()
      .map_err(|error| format!("The scene's geometry cannot be read: {error}"))
      .and_then(|mesh| SimulatedWorld::build(steam, &mesh))
    {
      Ok(world) => world,
      Err(reason) => {
        log::warn!("The scene's walls shape no sound: {reason}");
        shared.publish(SceneAcousticsState::new(SceneGeometryState::Failed(reason)));

        return;
      }
    };
    let geometry: SceneGeometryState = SceneGeometryState::Ready {
      triangles: world.triangles,
      build_time: started.elapsed(),
    };

    log::info!(
      "The acoustics' scene holds {} triangles, built in {:?}",
      world.triangles,
      started.elapsed()
    );

    let mut last: SceneAcousticsState = SceneAcousticsState::new(geometry);
    let mut reflected_at: Option<Instant> = None;

    shared.publish(last.clone());

    while let Some(input) = shared.wait() {
      let run_started: Instant = Instant::now();

      last.sources = world.run_direct(&input);
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
}

impl Drop for SceneSimulation {
  fn drop(&mut self) {
    self.shared.lock().is_closed = true;
    self.shared.wake.notify_one();
  }
}

impl SimulationShared {
  fn lock(&self) -> MutexGuard<'_, SharedState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
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

impl SimulatedWorld {
  fn build(steam: &SteamAudio, mesh: &AudioSceneMesh) -> Result<Self, String> {
    if !mesh.is_valid() {
      return Err(String::from(
        "The scene's geometry names vertices or materials it does not hold",
      ));
    }

    let context = steam.get_context();
    let failed = |what: &str, error: &dyn std::fmt::Display| format!("Steam Audio's {what} is not made: {error}");
    let mut scene: Scene = Scene::try_new(context).map_err(|error| failed("scene", &error))?;
    let vertices: Vec<Point> = mesh
      .vertices
      .iter()
      .map(|vertex| SteamSpace::to_point(*vertex))
      .collect();
    // Flipping `z` mirrors the space, so two corners swap to keep each face's front where the game has it.
    let triangles: Vec<Triangle> = mesh
      .triangles
      .iter()
      .map(|[a, b, c]| Triangle::new(*a as i32, *c as i32, *b as i32))
      .collect();
    let material_indices: Vec<usize> = mesh.material_indices.iter().map(|index| *index as usize).collect();
    let materials: Vec<Material> = mesh
      .materials
      .iter()
      .map(|material| Material {
        absorption: material.absorption,
        scattering: material.scattering,
        transmission: material.transmission,
      })
      .collect();

    if !triangles.is_empty() {
      let static_mesh = StaticMesh::try_new(
        &scene,
        &StaticMeshSettings {
          vertices: &vertices,
          triangles: &triangles,
          material_indices: &material_indices,
          materials: &materials,
        },
      )
      .map_err(|error| failed("mesh", &error))?;

      scene.add_static_mesh(static_mesh);
    }

    scene.commit();

    let settings = SteamAudio::get_audio_settings(AudioMixer::DEFAULT_OUTPUT_RATE);
    let mut direct: DirectSimulator = Simulator::try_new(
      context,
      &SimulationSettings::new(&settings).with_direct(DirectSimulationSettings {
        max_num_occlusion_samples: SceneSimulation::OCCLUSION_SAMPLES,
      }),
    )
    .map_err(|error| failed("direct simulator", &error))?;
    let mut reverb: ReverbSimulator = Simulator::try_new(
      context,
      &SimulationSettings::new(&settings).with_reflections(ParametricSettings {
        max_num_rays: SceneSimulation::REVERB_RAYS,
        num_diffuse_samples: 32,
        max_duration: MixerReverb::MAX_DURATION,
        max_num_sources: 1,
        num_threads: 2,
        max_order: 0,
      }),
    )
    .map_err(|error| failed("reverb simulator", &error))?;

    direct.set_scene(&scene);
    direct.commit();
    reverb.set_scene(&scene);

    let reverb_source: ReverbSource = Source::try_new(&reverb).map_err(|error| failed("reverb source", &error))?;

    reverb.add_source(&reverb_source);
    reverb.commit();

    Ok(Self {
      _scene: scene,
      direct,
      sources: HashMap::new(),
      reverb,
      reverb_source,
      triangles: triangles.len() as u32,
    })
  }

  /// Each source's way to the listener, a source the simulator cannot hear left unobstructed.
  fn run_direct(&mut self, input: &SimulationInput) -> HashMap<u64, SourceAcoustics> {
    let mut is_changed: bool = false;

    self.sources.retain(|id, source| {
      let is_kept: bool = input.sources.iter().any(|it| it.id == *id);

      if !is_kept {
        self.direct.remove_source(source);
        is_changed = true;
      }

      is_kept
    });

    for simulated in &input.sources {
      if self.sources.contains_key(&simulated.id) {
        continue;
      }

      match Source::try_new(&self.direct) {
        Ok(source) => {
          self.direct.add_source(&source);
          self.sources.insert(simulated.id, source);
          is_changed = true;
        }
        Err(error) => log::warn!("A sound's way through the walls is not simulated: {error}"),
      }
    }

    if is_changed {
      self.direct.commit();
    }

    let parameters = SimulationParameters::new().with_direct(
      DirectSimulationParameters::new().with_occlusion(
        Occlusion::new(OcclusionAlgorithm::Volumetric {
          radius: SceneSimulation::OCCLUSION_RADIUS,
          num_occlusion_samples: SceneSimulation::OCCLUSION_SAMPLES,
        })
        .with_transmission(TransmissionParameters {
          num_transmission_rays: SceneSimulation::TRANSMISSION_RAYS,
        }),
      ),
    );

    for simulated in &input.sources {
      if let Some(source) = self.sources.get(&simulated.id) {
        let inputs = SimulationInputs {
          source: SteamSpace::to_source(simulated.position),
          parameters: parameters.clone(),
        };

        if let Err(error) = source.set_direct_inputs(&inputs) {
          log::warn!("A sound's way through the walls is not simulated: {error:?}");
        }
      }
    }

    if let Err(error) = self
      .direct
      .set_shared_direct_inputs(&SimulationSharedInputs::new(SteamSpace::to_listener(&input.listener)).with_direct())
    {
      log::warn!("The sounds' ways through the walls are not simulated: {error:?}");

      return HashMap::new();
    }

    self.direct.run_direct();

    self
      .sources
      .iter()
      .filter_map(|(id, source)| {
        let outputs = source.get_direct_outputs().ok()?;
        let transmission: [f32; 3] = match outputs.transmission {
          Some(Transmission::FrequencyDependent(bands) | Transmission::FrequencyIndependent(bands)) => bands.0,
          None => [1.0; 3],
        };

        Some((
          *id,
          SourceAcoustics {
            occlusion: outputs.occlusion.unwrap_or(1.0),
            transmission,
          },
        ))
      })
      .collect()
  }

  /// Each band's reverb time around `listener`, none where the simulation fails.
  fn run_reflections(&mut self, listener: &AudioListener) -> Option<[f32; 3]> {
    let at: CoordinateSystem = SteamSpace::to_listener(listener);
    let inputs = SimulationInputs {
      source: at,
      parameters: SimulationParameters::new().with_reflections(ParametricParameters::default()),
    };
    let shared = SimulationSharedInputs::new(at).with_reflections(ReflectionsSharedInputs {
      num_rays: SceneSimulation::REVERB_RAYS,
      num_bounces: SceneSimulation::REVERB_BOUNCES,
      duration: MixerReverb::MAX_DURATION,
      order: 0,
      irradiance_min_distance: 1.0,
    });

    self.reverb_source.set_reflections_inputs(&inputs).ok()?;
    self.reverb.set_shared_reflections_inputs(&shared).ok()?;
    self.reverb.run_reflections().ok()?;

    let outputs = self.reverb_source.get_outputs_subset::<(), Reflections, ()>().ok()?;
    // SAFETY: the outputs hold the source's results until they are dropped, after this read.
    let times: [f32; 3] = unsafe { (*outputs.raw_ptr()).reflections.reverbTimes };

    times.iter().all(|time| time.is_finite()).then_some(times)
  }
}
