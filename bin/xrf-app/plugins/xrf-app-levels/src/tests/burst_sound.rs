//! A burst's sound: the scene's frames a burst draws, played on the burst's clock into `sound.wav`, the same for two
//! bursts from one time, each sound from the frame it starts on.

use std::collections::HashMap;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use xrf_audio::{
  AudioAssetSource, AudioBus, AudioEngineParts, AudioEngineSettings, AudioEvent, AudioListener, AudioSceneEmitter,
  AudioSceneFrame, AudioSceneGeometry, AudioSceneMesh, AudioSceneRules, AudioServer,
};
use xrf_error::XrfResult;
use xrf_renderer::RenderBurst;
use xrf_test_utils::GeneratedResources;

use crate::viewer::burst::{BurstSound, BurstSoundFrame, BurstSoundOutcome, BurstSoundTee};

/// The viewer's defaults but for fewer frames: 1/120 s apart from 600 s after a lead-in of 60.
const BURST: RenderBurst = RenderBurst {
  at: 600.0,
  step: 1.0 / 120.0,
  lead_in: 60,
  frames: 48,
  samples: None,
};

/// The level's sound clock at the burst's first frame: it runs on its own, begun as the level opened, ahead of the
/// picture's that the burst's start is given on.
const SOUND_AT: f64 = 601.9512;

/// The sound's clock at the burst's frame `index`, the lead-in's counted.
fn at(index: u32) -> Duration {
  Duration::from_secs_f64(SOUND_AT + f64::from(index) * f64::from(BURST.step))
}

/// Sounds held in memory by reference.
struct Sounds(HashMap<String, Arc<[u8]>>);

impl AudioAssetSource for Sounds {
  fn get_sound_scope(&self) -> &str {
    "burst"
  }

  fn read_sound(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(self.0.get(reference).map(|bytes| bytes.to_vec()))
  }
}

/// A scene with no walls.
struct NoWalls;

impl AudioSceneGeometry for NoWalls {
  fn read_mesh(&self) -> XrfResult<AudioSceneMesh> {
    Ok(AudioSceneMesh::default())
  }
}

/// A 16-bit mono WAV file of `samples` at 48 kHz.
fn wav(samples: &[i16]) -> Arc<[u8]> {
  let length: u32 = (samples.len() * 2) as u32;
  let mut bytes: Vec<u8> = Vec::new();

  bytes.extend_from_slice(b"RIFF");
  bytes.extend_from_slice(&(36 + length).to_le_bytes());
  bytes.extend_from_slice(b"WAVEfmt ");
  bytes.extend_from_slice(&16u32.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&1u16.to_le_bytes());
  bytes.extend_from_slice(&48_000u32.to_le_bytes());
  bytes.extend_from_slice(&96_000u32.to_le_bytes());
  bytes.extend_from_slice(&2u16.to_le_bytes());
  bytes.extend_from_slice(&16u16.to_le_bytes());
  bytes.extend_from_slice(b"data");
  bytes.extend_from_slice(&length.to_le_bytes());
  samples
    .iter()
    .for_each(|sample| bytes.extend_from_slice(&sample.to_le_bytes()));

  bytes.into()
}

/// A second of a 440 Hz tone, and a tenth of a second at a constant level, its first frame heard alone.
fn sounds() -> Arc<dyn AudioAssetSource> {
  let tone: Vec<i16> = (0..48_000)
    .map(|frame| ((TAU * 440.0 * frame as f32 / 48_000.0).sin() * 12_000.0) as i16)
    .collect();

  Arc::new(Sounds(HashMap::from([
    (String::from("tone"), wav(&tone)),
    (String::from("click"), wav(&[12_000; 4_800])),
  ])))
}

fn emitter(id: u64, sound: &str, position: [f32; 3]) -> AudioSceneEmitter {
  AudioSceneEmitter {
    id,
    sound: Arc::from(sound),
    bus: AudioBus::Ambience,
    position,
    // Started before the burst, so it plays from where its clock has it.
    start: Duration::from_secs(590),
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

/// The sound a burst records of frames `emitters` gives each moment, sent as the world sends them, sixty a second,
/// into the folder `name`; answers what was written and the file's bytes.
fn record(name: &str, emitters: impl Fn(Duration) -> Vec<AudioSceneEmitter>) -> (BurstSoundOutcome, Vec<u8>) {
  let folder: PathBuf = GeneratedResources::path(&format!("burst_sound/{name}"));
  let AudioEngineParts { engine, .. } = AudioEngineParts::new(AudioEngineSettings::default());
  let server: AudioServer = AudioServer::start_on(engine, None, |_: AudioEvent| {}).expect("the server starts");
  let tee: BurstSoundTee = BurstSoundTee::default();
  let sound: BurstSound = BurstSound::start(&BURST, &server, &tee).expect("the sound starts");
  let (assets, geometry): (Arc<dyn AudioAssetSource>, Arc<dyn AudioSceneGeometry>) = (sounds(), Arc::new(NoWalls));

  let _ = fs::remove_dir_all(&folder);
  fs::create_dir_all(&folder).expect("the folder to be made");

  for index in (0..BURST.lead_in + BURST.frames).step_by(2) {
    let time: Duration = at(index);

    tee.send(BurstSoundFrame {
      frame: Box::new(AudioSceneFrame {
        scene: 1,
        time,
        rules: AudioSceneRules::Vanilla,
        is_audible: true,
        listener: AudioListener {
          position: [(time - at(0)).as_secs_f32(), 0.0, 0.0],
          ..AudioListener::default()
        },
        heard: [true; AudioBus::COUNT],
        emitters: emitters(time),
      }),
      assets: Arc::clone(&assets),
      geometry: Arc::clone(&geometry),
    });
  }

  let outcome: BurstSoundOutcome = sound.write(&folder).expect("the sound to be written");
  let bytes: Vec<u8> = fs::read(&outcome.file).expect("the sound to be read back");

  (outcome, bytes)
}

/// The left channel of a `sound.wav`'s float samples.
fn lefts(bytes: &[u8]) -> Vec<f32> {
  bytes[58..]
    .as_chunks::<8>()
    .0
    .iter()
    .map(|pair| f32::from_le_bytes([pair[0], pair[1], pair[2], pair[3]]))
    .collect()
}

// Two bursts from one time write the same file to the byte: a tone either side of a moving listener, two as loud as
// each other, and a sound starting between frames.
#[test]
fn writes_the_same_sound_for_two_bursts_from_one_time() {
  let emitters = |_: Duration| {
    vec![
      emitter(1, "tone", [0.0, 0.0, 4.0]),
      emitter(2, "tone", [3.0, 0.0, 2.0]),
      emitter(3, "tone", [-3.0, 0.0, 2.0]),
      AudioSceneEmitter {
        start: at(0) + Duration::from_secs_f64(0.7123),
        is_looped: false,
        is_2d: true,
        ..emitter(4, "click", [0.0; 3])
      },
    ]
  };
  let (first, first_bytes) = record("first", emitters);
  let (second, second_bytes) = record("second", emitters);
  let frames: usize = (first_bytes.len() - 58) / 8;

  assert_eq!(first.rate, 48_000);
  assert_eq!(first.channels, 2);
  assert_eq!(&first_bytes[..4], b"RIFF");
  // 48 frames at 400 samples each, every frame's offset where its time falls.
  assert_eq!(frames, 48 * 400);
  assert_eq!(first.offsets.len(), 48);
  assert_eq!(first.offsets[1], 400);
  assert_eq!(first.offsets[47], 47 * 400);
  assert!(lefts(&first_bytes).iter().any(|sample| sample.abs() > 0.01));
  assert_eq!(first.offsets, second.offsets);
  assert!(first_bytes == second_bytes, "the two bursts' sounds differ");
}

// One burst records its sound at a time, and the next may once it is written or let go.
#[test]
fn refuses_a_second_bursts_sound_while_one_records() {
  let AudioEngineParts { engine, .. } = AudioEngineParts::new(AudioEngineSettings::default());
  let server: AudioServer = AudioServer::start_on(engine, None, |_: AudioEvent| {}).expect("the server starts");
  let tee: BurstSoundTee = BurstSoundTee::default();
  let first: BurstSound = BurstSound::start(&BURST, &server, &tee).expect("the sound starts");

  assert!(BurstSound::start(&BURST, &server, &tee).is_err());

  drop(first);

  assert!(BurstSound::start(&BURST, &server, &tee).is_ok());
}

/// The first sample of the left channel heard, of a click starting at `start`, named only by frames from `named` on.
fn first_heard(name: &str, start: Duration, named: Duration) -> Option<usize> {
  let (_, bytes) = record(name, |time| {
    (time >= named)
      .then(|| AudioSceneEmitter {
        start,
        is_looped: false,
        is_2d: true,
        ..emitter(1, "click", [0.0; 3])
      })
      .into_iter()
      .collect()
  });

  lefts(&bytes).iter().position(|sample| *sample != 0.0)
}

// A sound plays its first sample on the frame of the sound its start falls on: one the world starts with a frame, at
// that frame's time, and one it names ahead of its start, between two frames.
#[test]
fn lands_a_sound_on_the_sample_its_start_falls_on() {
  let between: Duration = at(0) + Duration::from_secs_f64(0.7123);

  // The file starts at the first frame read back, after the lead-in of 60.
  assert_eq!(first_heard("with-a-frame", at(86), at(86)), Some(26 * 400));
  assert_eq!(
    first_heard("between-frames", between, at(0)),
    Some(((0.7123 - 0.5) * 48_000.0_f64).round() as usize)
  );
}
