use std::collections::HashMap;

use audionimbus::{
  CoordinateSystem, DefaultRayTracer, Direct, DirectSimulationParameters, DirectSimulationSettings, Material,
  Occlusion, OcclusionAlgorithm, Parametric, ParametricParameters, ParametricSettings, Point, Reflections,
  ReflectionsSharedInputs, Scene, SimulationInputs, SimulationParameters, SimulationSettings, SimulationSharedInputs,
  Simulator, Source, StaticMesh, StaticMeshSettings, Transmission, TransmissionParameters, Triangle,
};

use crate::acoustics::scene::simulated_source::SimulatedSource;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;
use crate::acoustics::steam::mixer_reverb::MixerReverb;
use crate::acoustics::steam::steam_audio::SteamAudio;
use crate::acoustics::steam::steam_space::SteamSpace;
use crate::host::audio_scene_mesh::AudioSceneMesh;
use crate::mixer::audio_mixer::AudioMixer;
use xrf_audio_scene::AudioListener;

type DirectSimulator = Simulator<DefaultRayTracer, Direct>;
type DirectSource = Source<Direct>;
type ReverbSimulator = Simulator<DefaultRayTracer, (), Reflections, (), Parametric>;
type ReverbSource = Source<(), Reflections, (), Parametric>;

/// Steam Audio's scene of a scene's walls and the simulators tracing it: each sound's way to the listener through the
/// walls, and the listener's reverb from the rays it sends around.
pub(crate) struct SimulatedWorld {
  _scene: Scene,
  direct: DirectSimulator,
  sources: HashMap<u64, DirectSource>,
  reverb: ReverbSimulator,
  reverb_source: ReverbSource,
  triangles: u32,
}

impl SimulatedWorld {
  /// Points sampled around each sound: more makes a sound slide into occlusion rather than step.
  const OCCLUSION_SAMPLES: u32 = 16;

  /// How big a sound is taken to be for its occlusion, in metres.
  const OCCLUSION_RADIUS: f32 = 1.0;

  /// Metres a sound's way through the walls is heard from above where it is placed. A level places a zone or an object
  /// by its foot, on the ground or a little sunk into it, and a sound whose centre is under a surface is hidden from
  /// every listener: a smaller radius or a single ray hears no more of it, while half a metre up it reaches a listener in
  /// the open as it should, without lifting it through a ceiling it stands under.
  const OCCLUSION_LIFT: f32 = 0.5;

  /// Walls in the way whose transmission is counted, nearest the listener first.
  const TRANSMISSION_RAYS: u32 = 4;

  /// Rays sent from the listener, each reflected at most this many times, to hear its reverb.
  const REVERB_RAYS: u32 = 4096;
  const REVERB_BOUNCES: u32 = 16;

  /// Steam Audio's scene of `mesh`, its reverb traced on `reverb_threads` threads.
  pub(crate) fn build(steam: &SteamAudio, mesh: &AudioSceneMesh, reverb_threads: u32) -> Result<Self, String> {
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
        max_num_occlusion_samples: Self::OCCLUSION_SAMPLES,
      }),
    )
    .map_err(|error| failed("direct simulator", &error))?;
    let mut reverb: ReverbSimulator = Simulator::try_new(
      context,
      &SimulationSettings::new(&settings).with_reflections(ParametricSettings {
        max_num_rays: Self::REVERB_RAYS,
        num_diffuse_samples: 32,
        max_duration: MixerReverb::MAX_DURATION,
        max_num_sources: 1,
        num_threads: reverb_threads,
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

  pub(crate) fn get_triangles(&self) -> u32 {
    self.triangles
  }

  /// Each source's way to `listener`, a source the simulator cannot hear left unobstructed.
  pub(crate) fn run_direct(
    &mut self,
    listener: &AudioListener,
    sources: &[SimulatedSource],
  ) -> HashMap<u64, SourceAcoustics> {
    let mut is_changed: bool = false;

    self.sources.retain(|id, source| {
      let is_kept: bool = sources.iter().any(|it| it.id == *id);

      if !is_kept {
        self.direct.remove_source(source);
        is_changed = true;
      }

      is_kept
    });

    for simulated in sources {
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
          radius: Self::OCCLUSION_RADIUS,
          num_occlusion_samples: Self::OCCLUSION_SAMPLES,
        })
        .with_transmission(TransmissionParameters {
          num_transmission_rays: Self::TRANSMISSION_RAYS,
        }),
      ),
    );

    for simulated in sources {
      if let Some(source) = self.sources.get(&simulated.id) {
        let [x, y, z] = simulated.position;
        let inputs = SimulationInputs {
          source: SteamSpace::to_source([x, y + Self::OCCLUSION_LIFT, z]),
          parameters: parameters.clone(),
        };

        if let Err(error) = source.set_direct_inputs(&inputs) {
          log::warn!("A sound's way through the walls is not simulated: {error:?}");
        }
      }
    }

    if let Err(error) = self
      .direct
      .set_shared_direct_inputs(&SimulationSharedInputs::new(SteamSpace::to_listener(listener)).with_direct())
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
  pub(crate) fn run_reflections(&mut self, listener: &AudioListener) -> Option<[f32; 3]> {
    let at: CoordinateSystem = SteamSpace::to_listener(listener);
    let inputs = SimulationInputs {
      source: at,
      parameters: SimulationParameters::new().with_reflections(ParametricParameters::default()),
    };
    let shared = SimulationSharedInputs::new(at).with_reflections(ReflectionsSharedInputs {
      num_rays: Self::REVERB_RAYS,
      num_bounces: Self::REVERB_BOUNCES,
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
