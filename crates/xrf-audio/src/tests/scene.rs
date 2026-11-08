//! A scene on the control side: the one loudness model and its distance curve, the scene's clock on the mixer's, and
//! an offline session playing emitters, sharing targets and refusing sounds.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::acoustics::distance_curve::DistanceCurve;
use crate::acoustics::listener_space::ListenerSpace;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;
use crate::render::bank::banked_sound::BankedSound;
use crate::render::bank::sound_bank::SoundBank;
use crate::render::emitter_loudness::EmitterLoudness;
use crate::render::scene::scene_clock::SceneClock;
use crate::tests::fixtures::{MemorySounds, RATE, rms_left, wav};
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
    offset: Duration::ZERO,
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
  assert_eq!(clock.get_time(u64::from(RATE), RATE), Some(Duration::from_secs(11)));

  // A frame 60 ms late, as a stalled render thread sends it, moves nothing.
  assert!(!clock.follow(Duration::from_secs(11), u64::from(RATE) + 2880, RATE));
  assert_eq!(clock.get_time(u64::from(RATE), RATE), Some(Duration::from_secs(11)));

  // A clock stepped 150 ms is anchored again.
  assert!(clock.follow(Duration::from_millis(11_150), u64::from(RATE), RATE));
  assert_eq!(
    clock.get_time(u64::from(RATE), RATE),
    Some(Duration::from_millis(11_150))
  );
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

// An emitter played once from 0.9 s into its second-long sound is over a tenth of a second after its start, as one
// played from its first frame is not.
#[test]
fn plays_an_emitter_from_its_offset_into_its_sound() {
  let heard_from = |offset: Duration| -> f32 {
    let (mut session, mut inbox) = session();
    let played: AudioSceneEmitter = AudioSceneEmitter {
      is_looped: false,
      offset,
      ..emitter(1, "ambient\\tone")
    };

    assert!(
      inbox
        .send(frame(Duration::ZERO, AudioSceneRules::Vanilla, vec![played]))
        .is_ok()
    );

    let heard: Vec<f32> = session.render(RATE as usize / 2);

    rms_left(&heard[heard.len() / 2..])
  };

  assert!(heard_from(Duration::ZERO) > 0.05);
  assert!(heard_from(Duration::from_millis(900)) < 1e-4);
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

// A looped sound parting at 20 m/s plays about 5% slow for its Doppler shift, and keeps its voice: a voice is reckoned
// at the rate it was set to, so a pitch shift never reads as a starved stream.
#[test]
fn keeps_the_voice_of_a_sound_shifted_by_its_motion() {
  let (mut session, mut inbox) = session();

  for step in 0..80u32 {
    let time: Duration = Duration::from_millis(u64::from(step) * 50);
    let parting: AudioSceneEmitter = AudioSceneEmitter {
      position: [0.0, 0.0, 4.0 + 20.0 * time.as_secs_f32()],
      max_distance: 500.0,
      ..emitter(1, "ambient\\tone")
    };

    while let Some(spent) = inbox.take_spent() {
      drop(spent);
    }

    assert!(inbox.send(frame(time, AudioSceneRules::Vanilla, vec![parting])).is_ok());
    session.render(RATE as usize / 20);
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

/// A session holding the tone and `click`, a tenth of a second at a constant level, so its first frame is heard alone.
fn clicking_session() -> (OfflineSession, AudioSceneInbox) {
  let mut session: OfflineSession = OfflineSession::new(AudioEngineSettings::default(), RATE);
  let inbox: AudioSceneInbox = session.open_scene(
    Arc::new(MemorySounds(HashMap::from([
      (String::from("ambient\\tone"), tone()),
      (String::from("click"), wav(RATE, 1, &[16_000; RATE as usize / 10])),
    ]))),
    None,
  );

  (session, inbox)
}

/// Plays `frames` of the scene `step` seconds apart from nought, frame `index` naming `emitters(index)` and sent `lead`
/// mixer frames ahead of its time, and answers everything rendered up to the last frame's end.
fn play_ahead(
  session: &mut OfflineSession,
  inbox: &mut AudioSceneInbox,
  frames: u32,
  step: f64,
  lead: u64,
  emitters: impl Fn(u32) -> Vec<AudioSceneEmitter>,
) -> Vec<f32> {
  let at = |index: u32| (f64::from(index) * step * f64::from(RATE)).round() as u64;
  let mut heard: Vec<f32> = Vec::new();

  for index in 0..frames {
    let sent_at: u64 = at(index).saturating_sub(lead).max(heard.len() as u64 / 2);

    heard.extend(session.render((sent_at - heard.len() as u64 / 2) as usize));

    while inbox.take_spent().is_some() {}

    assert!(
      inbox
        .send(frame(
          Duration::from_secs_f64(f64::from(index) * step),
          AudioSceneRules::Vanilla,
          emitters(index),
        ))
        .is_ok()
    );
  }

  heard.extend(session.render((at(frames) - heard.len() as u64 / 2) as usize));
  heard
}

// A sound starting between two ticks and inside a mixer block plays its first frame on the frame its start falls on,
// the scene's frames sent as far ahead as the session asks.
#[test]
fn lands_a_start_inside_a_block_on_its_frame() {
  let (mut session, mut inbox) = clicking_session();
  let lead: u64 = session.get_lead_frames();
  // A second in, 37 frames into the 188th block and 37 frames past a tick.
  let start: u64 = u64::from(RATE) + 37;
  let click: AudioSceneEmitter = AudioSceneEmitter {
    is_2d: true,
    is_looped: false,
    start: Duration::from_secs_f64(start as f64 / f64::from(RATE)),
    ..emitter(1, "click")
  };
  let heard: Vec<f32> = play_ahead(&mut session, &mut inbox, 90, 1.0 / 60.0, lead, |_| vec![click.clone()]);
  let first: Option<usize> = heard.iter().step_by(2).position(|sample| *sample != 0.0);

  assert_eq!(first, Some(start as usize));
}

// The same frames sent the same way give the same samples to the bit: equal emitters rank by their ids however they
// are held, a sound starting meanwhile lands on its frame, and the session's ticks follow its own frames.
#[test]
fn plays_the_same_frames_the_same_to_the_bit() {
  let listen = || {
    let (mut session, mut inbox) = clicking_session();
    let lead: u64 = session.get_lead_frames();

    play_ahead(&mut session, &mut inbox, 120, 1.0 / 120.0, lead, |index| {
      let mut emitters: Vec<AudioSceneEmitter> = (0..12)
        .map(|id| AudioSceneEmitter {
          // Pairs as loud as each other, either side of the listener.
          position: [if id % 2 == 0 { 3.0 } else { -3.0 }, 0.0, 2.0 + (id / 2) as f32],
          volume: 0.5,
          ..emitter(id + 1, "ambient\\tone")
        })
        .collect();

      emitters.push(AudioSceneEmitter {
        position: [f32::from(index as u16) * 0.05, 0.0, 6.0],
        ..emitter(20, "ambient\\tone")
      });
      emitters.push(AudioSceneEmitter {
        is_2d: true,
        is_looped: false,
        start: Duration::from_secs_f64(0.6123),
        ..emitter(21, "click")
      });

      emitters
    })
  };
  let first: Vec<f32> = listen();
  let second: Vec<f32> = listen();

  assert!(rms_left(&first) > 0.01);
  assert_eq!(first.len(), second.len());
  assert!(
    first.iter().zip(&second).all(|(a, b)| a.to_bits() == b.to_bits()),
    "the two renders differ"
  );
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
