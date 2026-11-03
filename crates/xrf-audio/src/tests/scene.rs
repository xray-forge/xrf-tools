//! A scene on the control side: each engine's attenuation and smoothing, the scene's clock on the mixer's, and an
//! offline session playing emitters, sharing targets and refusing sounds.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use xrf_error::XrfResult;

use crate::render::emitter_loudness::EmitterLoudness;
use crate::render::scene_clock::SceneClock;
use crate::render::sound_bank::{BankedSound, SoundBank};
use crate::tests::fixtures::{RATE, rms_left, wav};
use crate::{
  AudioAssetSource, AudioBus, AudioEngineSettings, AudioListener, AudioSceneEmitter, AudioSceneFrame, AudioSceneInbox,
  AudioSceneReport, AudioSceneRules, OfflineSession,
};

/// Sounds held in memory by reference.
struct MemorySounds(HashMap<String, Arc<[u8]>>);

impl AudioAssetSource for MemorySounds {
  fn get_sound_scope(&self) -> &str {
    "memory"
  }

  fn read_sound(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(self.0.get(reference).map(|bytes| bytes.to_vec()))
  }
}

/// A second of a 440 Hz mono sine at half scale, as 16-bit PCM.
fn tone() -> Arc<[u8]> {
  let samples: Vec<i16> = (0..RATE)
    .map(|frame| ((std::f32::consts::TAU * 440.0 * frame as f32 / RATE as f32).sin() * 16_000.0) as i16)
    .collect();

  wav(RATE, 1, &samples)
}

fn emitter(id: u64, sound: &str) -> AudioSceneEmitter {
  AudioSceneEmitter {
    id,
    sound: Arc::from(sound),
    bus: AudioBus::Ambience,
    position: [0.0, 0.0, 4.0],
    start: Duration::ZERO,
    stop: None,
    is_looped: true,
    is_2d: false,
    volume: 1.0,
    frequency: 1.0,
    min_distance: 2.0,
    max_distance: 50.0,
    base_volume: 1.0,
    is_occluded: true,
    has_reverb: true,
  }
}

fn frame(time: Duration, rules: AudioSceneRules, emitters: Vec<AudioSceneEmitter>) -> Box<AudioSceneFrame> {
  Box::new(AudioSceneFrame {
    scene: 7,
    time,
    rules,
    is_audible: true,
    listener: AudioListener::default(),
    heard: [true; AudioBus::COUNT],
    emitters,
  })
}

fn session() -> (OfflineSession, AudioSceneInbox) {
  let mut session: OfflineSession = OfflineSession::new(AudioEngineSettings::default(), RATE);
  let inbox: AudioSceneInbox = session.open_scene(Arc::new(MemorySounds(HashMap::from([(
    String::from("ambient\\tone"),
    tone(),
  )]))));

  (session, inbox)
}

fn report(session: &OfflineSession) -> AudioSceneReport {
  session.describe_scene(7).expect("the scene is open")
}

// OpenXRay moves a tenth of the way each of its frames and leaves the distance to OpenAL's inverse-clamped model:
// `min / (min + 0.75 * (d - min))`, so `2 / 3.5` at four metres from a two-metre minimum.
#[test]
fn smooths_and_attenuates_as_openxray() {
  let mut played: AudioSceneEmitter = emitter(1, "a");
  let mut loudness: EmitterLoudness = EmitterLoudness::start(&played);

  played.volume = 0.8;
  loudness.update(AudioSceneRules::Vanilla, &played, Some(4.0), 1.0, 1.0 / 60.0, false);

  assert!(
    (loudness.get_smooth() - (0.9 + 0.1 * 0.8)).abs() < 1e-5,
    "{}",
    loudness.get_smooth()
  );
  assert!((loudness.get_gain(&played, Some(4.0)) - loudness.get_smooth() * 2.0 / 3.5).abs() < 1e-5);

  // Past its maximum distance it is culled at once.
  let culling = loudness.update(AudioSceneRules::Vanilla, &played, Some(60.0), 1.0, 1.0 / 60.0, false);

  assert!(!culling.is_audible);
  assert_eq!(loudness.get_smooth(), 0.0);
}

// Monolith assigns the volume: linear from the minimum distance to the maximum, capped by the volume rather than scaled
// by it, and OpenAL's model on top.
#[test]
fn attenuates_linearly_as_monolith() {
  let mut played: AudioSceneEmitter = emitter(1, "a");
  let mut loudness: EmitterLoudness = EmitterLoudness::start(&played);

  loudness.update(AudioSceneRules::Extended, &played, Some(26.0), 1.0, 1.0 / 60.0, false);

  assert!((loudness.get_smooth() - 0.5).abs() < 1e-5, "{}", loudness.get_smooth());
  assert!((loudness.get_gain(&played, Some(26.0)) - 0.5 * 2.0 / 20.0).abs() < 1e-5);

  played.volume = 0.3;
  loudness.update(AudioSceneRules::Extended, &played, Some(26.0), 1.0, 1.0 / 60.0, false);

  assert!((loudness.get_smooth() - 0.3).abs() < 1e-5, "{}", loudness.get_smooth());
}

// A stopping emitter fades out at ten a second, so it lets go of its sound a tenth of a second after the stop.
#[test]
fn fades_a_stopping_emitter_out_in_a_tenth_of_a_second() {
  let played: AudioSceneEmitter = emitter(1, "a");
  let mut loudness: EmitterLoudness = EmitterLoudness::start(&played);

  for _ in 0..5 {
    loudness.update(AudioSceneRules::Vanilla, &played, Some(4.0), 1.0, 0.01, true);
  }

  assert!(!loudness.is_faded());

  // A sixth step past the tenth of a second takes it below any rounding left.
  for _ in 0..6 {
    loudness.update(AudioSceneRules::Vanilla, &played, Some(4.0), 1.0, 0.01, true);
  }

  assert!(loudness.is_faded());
}

// The scene's clock is anchored on the mixer's once, then again only once a frame arrives more than 100 ms away from it.
#[test]
fn anchors_the_scene_clock_again_only_past_a_tenth_of_a_second() {
  let mut clock: SceneClock = SceneClock::default();

  assert!(clock.follow(Duration::from_secs(10), 0, RATE));
  assert_eq!(clock.get_time(RATE as u64, RATE), Some(Duration::from_secs(11)));

  // A frame 60 ms late, as a stalled render thread sends it, moves nothing.
  assert!(!clock.follow(Duration::from_secs(11), u64::from(RATE) + 2880, RATE));
  assert_eq!(clock.get_time(RATE as u64, RATE), Some(Duration::from_secs(11)));

  // A clock stepped 150 ms is anchored again.
  assert!(clock.follow(Duration::from_millis(11_150), u64::from(RATE), RATE));
  assert_eq!(clock.get_time(RATE as u64, RATE), Some(Duration::from_millis(11_150)));
}

#[test]
fn plays_an_emitter_where_the_scene_hears_it() {
  let (mut session, mut inbox) = session();

  assert!(
    inbox
      .send(frame(
        Duration::ZERO,
        AudioSceneRules::Vanilla,
        vec![emitter(1, "ambient\\tone")]
      ))
      .is_ok()
  );

  let heard: Vec<f32> = session.render(RATE as usize / 2);
  let report: AudioSceneReport = report(&session);

  assert_eq!(report.real_voices, 1);
  assert!(report.emitters[0].is_real && report.emitters[0].gain > 0.4);
  assert!(rms_left(&heard[heard.len() / 2..]) > 0.05);
}

// A clock stepped three times as fast as the mixer anchors again and again, and moves no playing voice: a voice is
// started over only where it lags its own pace.
#[test]
fn moves_no_playing_voice_for_a_stepped_clock() {
  let (mut session, mut inbox) = session();

  for step in 0..20 {
    let time: Duration = Duration::from_millis(step * 150);

    while let Some(spent) = inbox.take_spent() {
      drop(spent);
    }

    assert!(
      inbox
        .send(frame(time, AudioSceneRules::Vanilla, vec![emitter(1, "ambient\\tone")]))
        .is_ok()
    );
    session.render(2400);
  }

  let report: AudioSceneReport = report(&session);

  assert!(report.emitters[0].is_real);
  assert_eq!(report.emitters[0].restarts, 0);
}

// OpenXRay's 32 targets go to the loudest; the rest simulate.
#[test]
fn realizes_the_loudest_up_to_the_engines_targets() {
  let (mut session, mut inbox) = session();
  let emitters: Vec<AudioSceneEmitter> = (0..40)
    .map(|index| AudioSceneEmitter {
      position: [0.0, 0.0, 3.0 + index as f32],
      ..emitter(index + 1, "ambient\\tone")
    })
    .collect();

  assert!(
    inbox
      .send(frame(Duration::ZERO, AudioSceneRules::Vanilla, emitters))
      .is_ok()
  );
  session.render(4800);

  let report: AudioSceneReport = report(&session);
  let real: Vec<u64> = report.emitters.iter().filter(|it| it.is_real).map(|it| it.id).collect();

  assert_eq!(report.targets, 32);
  assert_eq!(real.len(), 32);
  assert!(real.iter().all(|id| *id <= 32), "{real:?}");
}

#[test]
fn refuses_a_sound_whose_file_is_missing() {
  let (mut session, mut inbox) = session();

  assert!(
    inbox
      .send(frame(
        Duration::ZERO,
        AudioSceneRules::Vanilla,
        vec![emitter(1, "ambient\\gone")]
      ))
      .is_ok()
  );
  session.render(4800);

  let report: AudioSceneReport = report(&session);

  assert_eq!(report.real_voices, 0);
  assert_eq!(report.refused.len(), 1);
  assert_eq!(report.refused[0].sound, "ambient\\gone");
}

// A scene no longer heard fades out over a quarter of a second and plays on, silent.
#[test]
fn fades_a_scene_no_longer_heard() {
  let (mut session, mut inbox) = session();

  assert!(
    inbox
      .send(frame(
        Duration::ZERO,
        AudioSceneRules::Vanilla,
        vec![emitter(1, "ambient\\tone")]
      ))
      .is_ok()
  );
  session.render(RATE as usize / 2);
  assert!(
    inbox
      .send(Box::new(AudioSceneFrame {
        is_audible: false,
        ..*frame(
          Duration::from_millis(500),
          AudioSceneRules::Vanilla,
          vec![emitter(1, "ambient\\tone")]
        )
      }))
      .is_ok()
  );

  let faded: Vec<f32> = session.render(RATE as usize / 2);

  assert!(rms_left(&faded[..960]) > 0.01);
  assert!(rms_left(&faded[faded.len() - 4800..]) < 1e-4);

  // Faded out, it lets its voice go and plays on simulated; heard again, it takes one where its clock says.
  let silent: AudioSceneReport = report(&session);

  assert_eq!(silent.real_voices, 0);
  assert!(!silent.emitters[0].is_real);
  assert!(
    inbox
      .send(frame(
        Duration::from_secs(1),
        AudioSceneRules::Vanilla,
        vec![emitter(1, "ambient\\tone")]
      ))
      .is_ok()
  );
  session.render(RATE as usize / 2);

  assert_eq!(report(&session).real_voices, 1);
}

// Past its budget the bank lets go of the sounds no emitter holds, the least recently asked first.
#[test]
fn lets_go_of_unheld_sounds_past_the_banks_budget() {
  let assets: Arc<dyn AudioAssetSource> = Arc::new(MemorySounds(HashMap::from([
    (String::from("a"), tone()),
    (String::from("b"), tone()),
    (String::from("c"), tone()),
  ])));
  let scope: Arc<str> = Arc::from("memory");
  // A second of mono float samples, and a little: room for one sound and part of another.
  let one: usize = RATE as usize * 4;
  let mut bank: SoundBank = SoundBank::inline_within(one + one / 2);
  let held: BankedSound = bank.get(&scope, &assets, &Arc::from("a"));

  bank.get(&scope, &assets, &Arc::from("b"));
  bank.get(&scope, &assets, &Arc::from("c"));

  // `a` is held by its emitter, so `b`, unheld and the oldest asked, went for `c`.
  assert_eq!(bank.get_size(), one * 2);
  assert!(matches!(held, BankedSound::Decoded(_)));
}
