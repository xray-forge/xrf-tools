//! The acoustics: sound played dry where Steam Audio is absent, and, with it, walls that occlude, doorways and materials
//! that let sound through, rooms whose reverb follows their size, and an audio thread that still allocates nothing.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::tests::fixtures::{MemorySounds, RATE, rms_left, tone};
use crate::{
  AudioAcoustics, AudioBus, AudioEngineSettings, AudioListener, AudioSceneEmitter, AudioSceneFrame, AudioSceneInbox,
  AudioSceneReport, AudioSceneRules, OfflineSession,
};

/// The scene's name in every frame.
const SCENE: u32 = 3;

fn emitter(id: u64, sound: &str, position: [f32; 3]) -> AudioSceneEmitter {
  AudioSceneEmitter {
    id,
    sound: Arc::from(sound),
    bus: AudioBus::Ambience,
    position,
    start: Duration::ZERO,
    offset: Duration::ZERO,
    stop: None,
    is_looped: true,
    is_2d: false,
    volume: 1.0,
    frequency: 1.0,
    min_distance: 2.0,
    max_distance: 80.0,
    base_volume: 1.0,
    has_reverb: true,
  }
}

fn frame(time: Duration, emitters: Vec<AudioSceneEmitter>) -> Box<AudioSceneFrame> {
  Box::new(AudioSceneFrame {
    scene: SCENE,
    time,
    rules: AudioSceneRules::Vanilla,
    is_audible: true,
    listener: AudioListener::default(),
    heard: [true; AudioBus::COUNT],
    emitters,
  })
}

/// Sends the scene's frame and renders `seconds` of it, ten milliseconds at a time as the world sends frames, answering
/// what was rendered.
fn play(
  session: &mut OfflineSession,
  inbox: &mut AudioSceneInbox,
  from: Duration,
  seconds: f32,
  emitters: &[AudioSceneEmitter],
) -> Vec<f32> {
  let steps: u32 = (seconds * 100.0).round() as u32;
  let mut heard: Vec<f32> = Vec::new();

  for step in 0..steps {
    while inbox.take_spent().is_some() {}

    let _ = inbox.send(frame(
      from + Duration::from_millis(u64::from(step) * 10),
      emitters.to_vec(),
    ));

    heard.extend(session.render(RATE as usize / 100));
  }

  heard
}

fn report(session: &OfflineSession) -> AudioSceneReport {
  session.describe_scene(SCENE).expect("the scene is open")
}

#[test]
fn plays_dry_and_says_why_where_steam_audio_is_absent() {
  let acoustics: AudioAcoustics = AudioAcoustics::load(Path::new("no-such-directory"));
  let mut session: OfflineSession = OfflineSession::with_acoustics(AudioEngineSettings::default(), RATE, acoustics);
  let mut inbox: AudioSceneInbox = session.open_scene(
    Arc::new(MemorySounds(HashMap::from([(String::from("tone"), tone(1.0))]))),
    None,
  );
  let heard: Vec<f32> = play(
    &mut session,
    &mut inbox,
    Duration::ZERO,
    0.5,
    &[emitter(1, "tone", [0.0, 0.0, 4.0])],
  );
  let report: AudioSceneReport = report(&session);

  assert!(report.acoustics.failure.is_some());
  assert!(report.emitters[0].is_real && !report.emitters[0].is_binaural);
  // Dry, at the curve's half at four metres from a two-metre minimum, panned to the centre.
  assert!(
    (report.emitters[0].gain - 0.5).abs() < 1e-3,
    "{}",
    report.emitters[0].gain
  );
  assert!(rms_left(&heard[heard.len() / 2..]) > 0.05);
}

#[cfg(feature = "steam-audio")]
mod steam {
  use std::collections::HashMap;
  use std::path::PathBuf;
  use std::sync::Arc;
  use std::time::{Duration, Instant};

  use assert_no_alloc::{assert_no_alloc, reset_violation_count, violation_count};
  use xrf_error::XrfResult;

  use super::{emitter, play, report};
  use crate::tests::fixtures::{MemorySounds, RATE, looped, rms_left, sine, tone};
  use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;
  use crate::{
    AudioAcoustics, AudioBus, AudioEngineParts, AudioEngineSettings, AudioMaterial, AudioPlay, AudioSceneEmitter,
    AudioSceneGeometry, AudioSceneInbox, AudioSceneMesh, AudioSceneReport, DecodedSound, OfflineRender, OfflineSession,
    SoundChannels,
  };

  /// Steam Audio as `cargo make fetch-sdks` placed it in the checkout, as the application ships it.
  fn checkout_acoustics() -> AudioAcoustics {
    let directory: PathBuf = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../.sdks/steam-audio"));
    let acoustics: AudioAcoustics = AudioAcoustics::load(&directory);

    assert!(
      acoustics.is_running(),
      "Steam Audio does not run from {}; run `cargo make fetch-sdks`: {:?}",
      directory.display(),
      acoustics.get_failure()
    );

    acoustics
  }

  struct Geometry(AudioSceneMesh);

  impl AudioSceneGeometry for Geometry {
    fn read_mesh(&self) -> XrfResult<AudioSceneMesh> {
      Ok(self.0.clone())
    }
  }

  /// Concrete, letting almost nothing through.
  const CONCRETE: AudioMaterial = AudioMaterial {
    absorption: [0.05, 0.07, 0.08],
    scattering: 0.05,
    transmission: [0.015, 0.002, 0.001],
  };

  /// Adds a quad of `corners`, in order around it, facing towards `inside`.
  fn quad(mesh: &mut AudioSceneMesh, corners: [[f32; 3]; 4], inside: [f32; 3], material: u32) {
    let base: u32 = mesh.vertices.len() as u32;
    let [a, b, c, _] = corners;
    let u: [f32; 3] = std::array::from_fn(|axis| b[axis] - a[axis]);
    let v: [f32; 3] = std::array::from_fn(|axis| c[axis] - a[axis]);
    let normal: [f32; 3] = [
      u[1] * v[2] - u[2] * v[1],
      u[2] * v[0] - u[0] * v[2],
      u[0] * v[1] - u[1] * v[0],
    ];
    let towards: f32 = (0..3).map(|axis| normal[axis] * (inside[axis] - a[axis])).sum();

    mesh.vertices.extend(corners);

    if towards >= 0.0 {
      mesh
        .triangles
        .extend([[base, base + 1, base + 2], [base, base + 2, base + 3]]);
    } else {
      mesh
        .triangles
        .extend([[base, base + 2, base + 1], [base, base + 3, base + 2]]);
    }

    mesh.material_indices.extend([material, material]);
  }

  /// A wall across `z`, filling `x` and `y` from -20 to 20 m but for the opening `hole`, `[x0, x1, y0, y1]`, if any.
  fn wall(z: f32, hole: Option<[f32; 4]>, material: AudioMaterial) -> AudioSceneMesh {
    let mut mesh: AudioSceneMesh = AudioSceneMesh {
      materials: vec![material],
      ..AudioSceneMesh::default()
    };
    let facing: [f32; 3] = [0.0, 0.0, 0.0];
    let mut add = |[x0, x1, y0, y1]: [f32; 4]| {
      quad(
        &mut mesh,
        [[x0, y0, z], [x1, y0, z], [x1, y1, z], [x0, y1, z]],
        facing,
        0,
      );
    };

    match hole {
      None => add([-20.0, 20.0, -20.0, 20.0]),
      Some([x0, x1, y0, y1]) => {
        add([-20.0, x0, -20.0, 20.0]);
        add([x1, 20.0, -20.0, 20.0]);
        add([x0, x1, -20.0, y0]);
        add([x0, x1, y1, 20.0]);
      }
    }

    mesh
  }

  /// A closed room `size` metres across, centred on the listener, its faces turned in.
  fn room([width, height, depth]: [f32; 3], material: AudioMaterial) -> AudioSceneMesh {
    let mut mesh: AudioSceneMesh = AudioSceneMesh {
      materials: vec![material],
      ..AudioSceneMesh::default()
    };
    let [x, y, z] = [width / 2.0, height / 2.0, depth / 2.0];
    let centre: [f32; 3] = [0.0; 3];

    for sign in [-1.0, 1.0] {
      quad(
        &mut mesh,
        [
          [sign * x, -y, -z],
          [sign * x, y, -z],
          [sign * x, y, z],
          [sign * x, -y, z],
        ],
        centre,
        0,
      );
      quad(
        &mut mesh,
        [
          [-x, sign * y, -z],
          [x, sign * y, -z],
          [x, sign * y, z],
          [-x, sign * y, z],
        ],
        centre,
        0,
      );
      quad(
        &mut mesh,
        [
          [-x, -y, sign * z],
          [x, -y, sign * z],
          [x, y, sign * z],
          [-x, y, sign * z],
        ],
        centre,
        0,
      );
    }

    mesh
  }

  /// What a listener hears of a tone: the scene's report, and half a second of it.
  struct Heard {
    report: AudioSceneReport,
    samples: Vec<f32>,
  }

  /// How a room rings: the reverb's level 0.3 to 0.8 s after a burst ends, and its reported decay times.
  struct Ring {
    tail: f32,
    reverb_times: [f32; 3],
  }

  /// A session over `mesh` with one looping tone, played until the simulation has heard the tone's way to the listener.
  fn hear_through(mesh: Option<AudioSceneMesh>, at: [f32; 3]) -> Heard {
    let mut session: OfflineSession =
      OfflineSession::with_acoustics(AudioEngineSettings::default(), RATE, checkout_acoustics());
    let mut inbox: AudioSceneInbox = session.open_scene(
      Arc::new(MemorySounds(HashMap::from([(String::from("tone"), tone(1.0))]))),
      mesh.map(|mesh| Arc::new(Geometry(mesh)) as Arc<dyn AudioSceneGeometry>),
    );
    let emitters: Vec<AudioSceneEmitter> = vec![emitter(1, "tone", at)];
    let deadline: Instant = Instant::now() + Duration::from_secs(20);
    let mut time: Duration = Duration::ZERO;

    // The simulation runs on its own thread in real time, while an offline render runs as fast as it can: wait for the
    // scene, then give the simulation a few of its runs with the tone in it.
    loop {
      play(&mut session, &mut inbox, time, 0.1, &emitters);
      time += Duration::from_millis(100);

      let report: AudioSceneReport = report(&session);
      let is_without_walls: bool = report.acoustics.geometry_failure.is_some();

      if is_without_walls || report.acoustics.triangles.is_some() {
        break;
      }

      assert!(
        Instant::now() < deadline,
        "the simulation never built the scene: {report:?}"
      );
      std::thread::sleep(Duration::from_millis(20));
    }

    for _ in 0..10 {
      play(&mut session, &mut inbox, time, 0.02, &emitters);
      time += Duration::from_millis(20);
      std::thread::sleep(Duration::from_millis(40));
    }

    let samples: Vec<f32> = play(&mut session, &mut inbox, time, 0.5, &emitters);

    Heard {
      report: report(&session),
      samples,
    }
  }

  #[test]
  fn muffles_a_sound_behind_a_wall() {
    let open: Heard = hear_through(None, [0.0, 0.0, 10.0]);
    let walled: Heard = hear_through(Some(wall(5.0, None, CONCRETE)), [0.0, 0.0, 10.0]);
    let emitter = &walled.report.emitters[0];

    assert!(open.report.acoustics.failure.is_none() && open.report.acoustics.geometry_failure.is_some());
    assert_eq!(walled.report.acoustics.triangles, Some(2));
    assert!(emitter.occlusion < 0.1, "{emitter:?}");
    assert!(emitter.transmission[0] < 0.05, "{emitter:?}");
    assert!(
      rms_left(&walled.samples) < rms_left(&open.samples) * 0.2,
      "{} against {}",
      rms_left(&walled.samples),
      rms_left(&open.samples)
    );
  }

  /// Ground at `y`, 40 m across about the listener, facing up.
  fn ground(y: f32) -> AudioSceneMesh {
    let mut mesh: AudioSceneMesh = AudioSceneMesh {
      materials: vec![CONCRETE],
      ..AudioSceneMesh::default()
    };

    quad(
      &mut mesh,
      [[-20.0, y, -20.0], [20.0, y, -20.0], [20.0, y, 20.0], [-20.0, y, 20.0]],
      [0.0, y + 1.0, 0.0],
      0,
    );

    mesh
  }

  // A level places a zone by its foot, a few centimetres into the ground as often as not: the ground it stands on hides
  // none of it from a listener across open ground, though Steam Audio hears nothing of a sound whose centre is under a
  // surface.
  #[test]
  fn hears_a_sound_standing_on_the_ground_across_open_ground() {
    let heard: Heard = hear_through(Some(ground(-1.7)), [0.0, -1.75, 10.0]);

    assert!(
      heard.report.emitters[0].occlusion > 0.9,
      "{:?}",
      heard.report.emitters[0]
    );
  }

  #[test]
  fn lets_a_sound_through_a_doorway() {
    let heard: Heard = hear_through(
      Some(wall(5.0, Some([-1.5, 1.5, -1.5, 2.5]), CONCRETE)),
      [0.0, 0.0, 10.0],
    );

    assert!(
      heard.report.emitters[0].occlusion > 0.8,
      "{:?}",
      heard.report.emitters[0]
    );
  }

  #[test]
  fn lets_through_what_a_walls_material_transmits() {
    let thin: AudioMaterial = AudioMaterial {
      transmission: [0.6, 0.4, 0.2],
      ..CONCRETE
    };
    let heard: Heard = hear_through(Some(wall(5.0, None, thin)), [0.0, 0.0, 10.0]);
    let transmission: [f32; 3] = heard.report.emitters[0].transmission;

    assert!(
      heard.report.emitters[0].occlusion < 0.1,
      "{:?}",
      heard.report.emitters[0]
    );
    assert!(
      transmission
        .iter()
        .zip([0.6, 0.4, 0.2])
        .all(|(heard, made)| (heard - made).abs() < 0.05),
      "{transmission:?}"
    );
  }

  /// How a room `size` metres across rings after a tenth of a second's burst at two metres, once its reverb has been
  /// simulated.
  fn ring(size: [f32; 3]) -> Ring {
    let absorbing: AudioMaterial = AudioMaterial {
      absorption: [0.3, 0.3, 0.3],
      scattering: 0.2,
      transmission: [0.0; 3],
    };
    let mut session: OfflineSession =
      OfflineSession::with_acoustics(AudioEngineSettings::default(), RATE, checkout_acoustics());
    let mut inbox: AudioSceneInbox = session.open_scene(
      Arc::new(MemorySounds(HashMap::from([(String::from("burst"), tone(0.1))]))),
      Some(Arc::new(Geometry(room(size, absorbing)))),
    );
    let burst_at: Duration = Duration::from_secs(30);
    let burst: Vec<AudioSceneEmitter> = vec![AudioSceneEmitter {
      start: burst_at,
      offset: Duration::ZERO,
      is_looped: false,
      ..emitter(1, "burst", [0.0, 0.0, 2.0])
    }];
    let deadline: Instant = Instant::now() + Duration::from_secs(30);
    let mut time: Duration = Duration::ZERO;

    loop {
      play(&mut session, &mut inbox, time, 0.1, &burst);
      time += Duration::from_millis(100);

      if report(&session).acoustics.reverb_times.iter().any(|time| *time > 0.0) {
        break;
      }

      assert!(Instant::now() < deadline, "the room's reverb was never simulated");
      std::thread::sleep(Duration::from_millis(20));
    }

    let heard: Vec<f32> = play(&mut session, &mut inbox, burst_at, 1.0, &burst);
    let frames = |seconds: f32| (seconds * RATE as f32) as usize * 2;

    Ring {
      tail: rms_left(&heard[frames(0.4)..frames(0.9)]),
      reverb_times: report(&session).acoustics.reverb_times,
    }
  }

  #[test]
  fn rings_longer_in_a_hall_than_in_a_small_room() {
    let small: Ring = ring([4.0, 3.0, 4.0]);
    let hall: Ring = ring([30.0, 15.0, 30.0]);

    assert!(
      hall.reverb_times[1] > small.reverb_times[1] * 2.0,
      "{:?} against {:?}",
      hall.reverb_times,
      small.reverb_times
    );
    assert!(
      hall.tail > small.tail * 4.0 && hall.tail > 1e-4,
      "{} against {}",
      hall.tail,
      small.tail
    );
  }

  #[test]
  fn renders_placed_voices_and_the_reverb_without_allocating() {
    let acoustics: AudioAcoustics = checkout_acoustics();
    let steam = acoustics.get_steam().expect("Steam Audio runs");
    let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 440.0, 0.5, SoundChannels::Mono));
    let AudioEngineParts { mut engine, mixer } = AudioEngineParts::with_acoustics(
      AudioEngineSettings {
        voices_count: 160,
        ..AudioEngineSettings::default()
      },
      acoustics.clone(),
    );
    let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
    let mut out: Vec<f32> = vec![0.0; 480 * 2];

    for index in 0..128 {
      let parameters: VoiceSpatialParameters = VoiceSpatialParameters {
        direction: [0.6, 0.0, -0.8],
        attenuation: 0.5,
        occlusion: 0.4,
        transmission: [0.3, 0.2, 0.1],
        reverb_send: 0.5,
        is_binaural: index < 64,
        ..VoiceSpatialParameters::default()
      };
      let effects = steam.create_voice_effects(RATE).expect("the effects are made");

      engine
        .play_placed(
          AudioPlay {
            gain: 0.01,
            ..AudioPlay::new(looped(&sound), AudioBus::World)
          },
          Box::new(effects),
          &parameters,
        )
        .expect("a voice starts");
    }

    engine.set_reverb_times([1.5, 1.2, 0.8]);
    // The voices are taken up and the reverb's buffers touched once before counting.
    render.render_into(&mut out);
    reset_violation_count();

    for _ in 0..100 {
      assert_no_alloc(|| render.render_into(&mut out));
    }

    assert_eq!(violation_count(), 0);
    assert!(rms_left(&out) > 0.0);
  }
}
