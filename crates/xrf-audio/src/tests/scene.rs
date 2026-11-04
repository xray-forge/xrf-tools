//! A scene on the control side: the one loudness model and its distance curve, the scene's clock on the mixer's, and
//! an offline session playing emitters, sharing targets and refusing sounds.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::acoustics::distance_curve::DistanceCurve;
use crate::acoustics::listener_space::ListenerSpace;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;
use crate::render::emitter_loudness::EmitterLoudness;
use crate::render::scene::scene_clock::SceneClock;
use crate::render::sound_bank::{BankedSound, SoundBank};
use crate::tests::fixtures::{MemorySounds, RATE, rms_left};
use crate::{
  AudioAssetSource, AudioBus, AudioEngineSettings, AudioListener, AudioSceneEmitter, AudioSceneFrame, AudioSceneInbox,
  AudioSceneReport, AudioSceneRules, OfflineSession,
};

/// A second of a 440 Hz mono sine.
fn tone() -> Arc<[u8]> {
  crate::tests::fixtures::tone(1.0)
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
  let inbox: AudioSceneInbox = session.open_scene(
    Arc::new(MemorySounds(HashMap::from([(String::from("ambient\\tone"), tone())]))),
    None,
  );

  (session, inbox)
}

fn report(session: &OfflineSession) -> AudioSceneReport {
  session.describe_scene(7).expect("the scene is open")
}

// Full up to the minimum distance, then one over the distance, eased to silence over the outer half of the range.
#[test]
fn falls_with_distance_by_one_curve() {
  assert_eq!(DistanceCurve::get_attenuation(2.0, 50.0, 1.0), 1.0);
  assert!((DistanceCurve::get_attenuation(2.0, 50.0, 4.0) - 0.5).abs() < 1e-6);
  assert!((DistanceCurve::get_attenuation(2.0, 50.0, 20.0) - 0.1).abs() < 1e-6);

  // Half way through the taper over the outer half of the range, from 26 m to 50 m, smoothstep leaves half.
  let middle: f32 = 50.0 - (48.0 * 0.5) / 2.0;

  assert!((DistanceCurve::get_attenuation(2.0, 50.0, middle) - 2.0 / middle * 0.5).abs() < 1e-5);
  assert_eq!(DistanceCurve::get_attenuation(2.0, 50.0, 50.0), 0.0);
  assert_eq!(DistanceCurve::get_attenuation(2.0, 50.0, f32::NAN), 0.0);
}

// A playing sound's walls ease toward each new simulation, about two thirds of the way in a tenth of a second, so a
// sound moving behind an edge fades rather than steps.
#[test]
fn eases_a_sounds_walls_toward_each_simulation() {
  let walled: SourceAcoustics = SourceAcoustics {
    occlusion: 0.0,
    transmission: [0.1; 3],
  };
  let mut heard: SourceAcoustics = SourceAcoustics::default();

  heard.approach(walled, 0.01);

  assert!((heard.occlusion - (-0.1f32).exp()).abs() < 1e-4, "{heard:?}");

  heard.approach(walled, 0.09);

  assert!((heard.occlusion - (-1.0f32).exp()).abs() < 1e-4, "{heard:?}");
  assert!(
    (heard.transmission[0] - (0.1 + 0.9 * (-1.0f32).exp())).abs() < 1e-4,
    "{heard:?}"
  );
}

// Volume times the curve for a placed sound whatever the engine; past its maximum distance it is culled at once.
#[test]
fn attenuates_every_engine_alike() {
  let mut played: AudioSceneEmitter = emitter(1, "a");
  let mut loudness: EmitterLoudness = EmitterLoudness::start();

  played.volume = 0.8;

  let culling = loudness.update(&played, Some(4.0), 1.0 / 60.0, false);

  assert!(culling.is_audible);
  assert!((culling.priority - 0.8 * 0.5).abs() < 1e-6, "{}", culling.priority);
  assert!((loudness.get_gain(&played) - 0.8).abs() < 1e-6);
  assert!((EmitterLoudness::get_attenuation(&played, Some(4.0)) - 0.5).abs() < 1e-6);

  let culling = loudness.update(&played, Some(60.0), 1.0 / 60.0, false);

  assert!(!culling.is_audible);
  assert_eq!(loudness.get_gain(&played), 0.0);
}

// A stopping emitter fades out at ten a second, so it lets go of its sound a tenth of a second after the stop.
#[test]
fn fades_a_stopping_emitter_out_in_a_tenth_of_a_second() {
  let played: AudioSceneEmitter = emitter(1, "a");
  let mut loudness: EmitterLoudness = EmitterLoudness::start();

  for _ in 0..5 {
    loudness.update(&played, Some(4.0), 0.01, true);
  }

  assert!(!loudness.is_faded());

  // A sixth step past the tenth of a second takes it below any rounding left.
  for _ in 0..6 {
    loudness.update(&played, Some(4.0), 0.01, true);
  }

  assert!(loudness.is_faded());
}

// A sound ahead is ahead in Steam Audio's listener space, `-z`; one to the right is `+x`, as the listener turns.
#[test]
fn places_a_sound_in_the_listeners_own_space() {
  let listener: AudioListener = AudioListener {
    position: [10.0, 0.0, 10.0],
    ..AudioListener::default()
  };
  let ahead: [f32; 3] = ListenerSpace::get_direction(&listener, [10.0, 0.0, 20.0]);
  let right: [f32; 3] = ListenerSpace::get_direction(&listener, [15.0, 0.0, 10.0]);
  let turned: AudioListener = AudioListener {
    forward: [1.0, 0.0, 0.0],
    ..listener
  };

  assert!((ahead[2] + 1.0).abs() < 1e-6, "{ahead:?}");
  assert!((right[0] - 1.0).abs() < 1e-6, "{right:?}");
  // Facing `+x`, the same sound is now straight ahead.
  assert!((ListenerSpace::get_direction(&turned, [15.0, 0.0, 10.0])[2] + 1.0).abs() < 1e-6);
}

// Closing in raises the pitch and parting lowers it; a camera's jump is held to a factor of one and a half.
#[test]
fn shifts_the_pitch_of_a_sound_closing_in_or_parting() {
  let listener: AudioListener = AudioListener::default();
  let at: [f32; 3] = [0.0, 0.0, 20.0];

  assert!(ListenerSpace::get_doppler(&listener, at, [0.0, 0.0, -30.0]) > 1.05);
  assert!(ListenerSpace::get_doppler(&listener, at, [0.0, 0.0, 30.0]) < 0.95);
  assert_eq!(ListenerSpace::get_doppler(&listener, at, [0.0; 3]), 1.0);

  let jumping: AudioListener = AudioListener {
    velocity: [0.0, 0.0, 5000.0],
    ..listener
  };

  assert!(ListenerSpace::get_doppler(&jumping, at, [0.0; 3]) <= 1.5);
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
