use std::sync::Arc;

use crate::tests::fixtures::{RATE, engine, frequency_left, looped, once, rms_left, sine};
use crate::{
  AudioBus, AudioEngineParts, AudioPlay, AudioRefusal, AudioVoiceId, AudioVoicePlayback, AudioVoiceState, DecodedSound,
  OfflineRender, SoundChannels, VoiceSource,
};

#[test]
fn plays_a_sine_through_each_bus_at_each_volume() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));

  for bus in AudioBus::ALL {
    for gain in [0.0, 0.25, 0.5, 1.0] {
      let AudioEngineParts { mut engine, mixer } = engine(4);
      let mut render: OfflineRender = OfflineRender::new(mixer, RATE);

      engine
        .play(AudioPlay::new(looped(&sound), bus))
        .expect("a voice starts");
      engine.set_bus_gain(bus, gain).expect("the bus takes its gain");
      render.render(4800);

      let measured: f32 = rms_left(&render.render(4800));
      let expected: f32 = 0.5 * gain / 2f32.sqrt();

      assert!(
        (measured - expected).abs() < 0.005,
        "{bus:?} at {gain}: {measured} against {expected}"
      );
    }
  }
}

#[test]
fn scales_every_bus_by_the_master_and_leaves_other_buses_out() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Stereo));
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);

  engine
    .play(AudioPlay::new(looped(&sound), AudioBus::Music))
    .expect("a voice starts");
  engine
    .set_bus_gain(AudioBus::Previews, 0.0)
    .expect("the bus takes its gain");
  engine.set_master_gain(0.5).expect("the master takes its gain");
  render.render(4800);

  let measured: f32 = rms_left(&render.render(4800));

  assert!((measured - 0.25 / 2f32.sqrt()).abs() < 0.005, "{measured}");
}

#[test]
fn resamples_a_sound_to_the_output_rate_and_its_playback_rate() {
  let sound: Arc<DecodedSound> = Arc::new(sine(22_050, 22_050, 440.0, 0.5, SoundChannels::Mono));

  for (rate, expected) in [(1.0, 440.0), (2.0, 880.0)] {
    let AudioEngineParts { mut engine, mixer } = engine(1);
    let mut render: OfflineRender = OfflineRender::new(mixer, RATE);

    engine
      .play(AudioPlay {
        rate,
        ..AudioPlay::new(looped(&sound), AudioBus::Ambience)
      })
      .expect("a voice starts");

    let measured: f32 = frequency_left(&render.render(RATE as usize), RATE);

    assert!((measured - expected).abs() <= 2.0, "rate {rate}: {measured} Hz");
  }
}

#[test]
fn ends_a_sound_played_once_and_frees_its_slot() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, 1000, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let first: AudioVoiceId = engine
    .play(AudioPlay::new(once(&sound), AudioBus::World))
    .expect("a voice starts");

  assert_eq!(engine.get_playback(first).state, AudioVoiceState::Starting);
  assert_eq!(
    engine.play(AudioPlay::new(once(&sound), AudioBus::World)),
    Err(AudioRefusal::VoicesExhausted)
  );

  render.render(512);
  assert_eq!(engine.get_playback(first).state, AudioVoiceState::Playing);

  let tail: Vec<f32> = render.render(1024);

  assert_eq!(engine.get_playback(first).state, AudioVoiceState::Ended);
  // The sound's 1000 frames end 488 frames into the second render.
  assert!(
    tail[488 * 2..].iter().all(|sample| *sample == 0.0),
    "silence after the end"
  );
  assert!(
    tail[..488 * 2].iter().any(|sample| *sample != 0.0),
    "sound before the end"
  );

  let second: AudioVoiceId = engine
    .play(AudioPlay::new(once(&sound), AudioBus::World))
    .expect("the slot is free");

  assert_eq!(second.slot, first.slot);
  assert_ne!(second.generation, first.generation);
  assert_eq!(engine.get_playback(first).state, AudioVoiceState::Ended);
}

#[test]
fn holds_a_paused_voice_in_place_and_ends_a_stopped_one_after_its_fade() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(looped(&sound), AudioBus::Music))
    .expect("a voice starts");

  render.render(4800);
  engine.pause(voice).expect("the voice pauses");
  render.render(480);

  let paused_at: u64 = engine.get_playback(voice).frame;

  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Paused);
  assert!(
    render.render(4800).iter().all(|sample| *sample == 0.0),
    "a paused voice is silent"
  );
  assert_eq!(engine.get_playback(voice).frame, paused_at);

  engine.resume(voice).expect("the voice resumes");
  render.render(4800);
  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Playing);
  assert_eq!(engine.get_playback(voice).frame, paused_at + 4800);

  engine.stop(voice).expect("the voice stops");
  render.render(480);
  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Ended);
}

#[test]
fn ignores_a_command_for_a_voice_whose_slot_was_reused() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, 100, 1000.0, 0.5, SoundChannels::Mono));
  let long: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let ended: AudioVoiceId = engine
    .play(AudioPlay::new(once(&sound), AudioBus::World))
    .expect("a voice starts");

  render.render(256);

  let playing: AudioVoiceId = engine
    .play(AudioPlay::new(looped(&long), AudioBus::World))
    .expect("the slot is free");

  engine.stop(ended).expect("the stale stop is sent");
  render.render(4800);
  assert_eq!(engine.get_playback(playing).state, AudioVoiceState::Playing);
}

#[test]
fn refuses_commands_while_no_buffer_takes_them() {
  let AudioEngineParts {
    mut engine,
    mixer: _mixer,
  } = AudioEngineParts::new(crate::AudioEngineSettings {
    voices_count: 1,
    commands_count: 2,
  });

  engine.set_master_gain(1.0).expect("the first command fits");
  engine.set_master_gain(1.0).expect("the second command fits");
  assert_eq!(engine.set_master_gain(1.0), Err(AudioRefusal::CommandsFull));
}

#[test]
fn follows_a_voice_s_gain_and_rate_from_the_next_buffer() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(looped(&sound), AudioBus::Ambience))
    .expect("a voice starts");

  render.render(480);
  engine.set_voice_gain(voice, 0.5);
  engine.set_voice_rate(voice, 2.0);
  render.render(480);

  let frame: u64 = engine.get_playback(voice).frame;
  let out: Vec<f32> = render.render(4800);

  assert!(
    (rms_left(&out) - 0.25 / 2f32.sqrt()).abs() < 0.005,
    "{}",
    rms_left(&out)
  );
  assert_eq!(engine.get_playback(voice).frame, frame + 9600);
}

/// Continuous controls never wait on the audio thread: with no device rendering, as many writes as a level's voices
/// send in many frames are all kept, the last of each winning.
#[test]
fn keeps_the_last_of_many_writes_while_no_buffer_renders() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = AudioEngineParts::new(crate::AudioEngineSettings {
    voices_count: 1,
    commands_count: 4,
  });
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(looped(&sound), AudioBus::World))
    .expect("a voice starts");

  for write in 0..100_000 {
    engine.set_voice_gain(voice, write as f32 / 100_000.0);
  }

  engine.set_voice_gain(voice, 0.25);
  render.render(4800);

  assert!((rms_left(&render.render(4800)) - 0.125 / 2f32.sqrt()).abs() < 0.005);
}

#[test]
fn never_writes_an_ended_voice_s_controls_into_the_next_voice_in_its_slot() {
  let short: Arc<DecodedSound> = Arc::new(sine(RATE, 100, 1000.0, 0.5, SoundChannels::Mono));
  let long: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let ended: AudioVoiceId = engine
    .play(AudioPlay::new(once(&short), AudioBus::World))
    .expect("a voice starts");

  render.render(256);

  let playing: AudioVoiceId = engine
    .play(AudioPlay::new(looped(&long), AudioBus::World))
    .expect("the slot is free");

  engine.set_voice_gain(ended, 0.0);
  render.render(4800);

  assert_eq!(playing.slot, ended.slot);
  assert!((rms_left(&render.render(4800)) - 0.5 / 2f32.sqrt()).abs() < 0.005);
}

#[test]
fn reports_where_a_voice_starts_before_the_audio_thread_takes_it_up() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, 10_000, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(
      VoiceSource::Decoded {
        sound,
        start_frame: 6000,
        is_looped: false,
      },
      AudioBus::Previews,
    ))
    .expect("a voice starts");

  assert_eq!(
    engine.get_playback(voice),
    AudioVoicePlayback {
      state: AudioVoiceState::Starting,
      frame: 6000,
    }
  );

  render.render(100);
  assert_eq!(engine.get_playback(voice).frame, 6100);
}

#[test]
fn plays_a_bad_rate_at_the_sound_s_own_and_a_bad_gain_as_silence() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));

  for rate in [0.0, -1.0, f32::NAN, f32::INFINITY] {
    let AudioEngineParts { mut engine, mixer } = engine(1);
    let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
    let voice: AudioVoiceId = engine
      .play(AudioPlay {
        rate,
        ..AudioPlay::new(once(&sound), AudioBus::World)
      })
      .expect("a voice starts");

    render.render(480);
    assert_eq!(engine.get_playback(voice).frame, 480, "rate {rate}");
  }

  for gain in [-1.0, f32::NAN, f32::INFINITY] {
    let AudioEngineParts { mut engine, mixer } = engine(1);
    let mut render: OfflineRender = OfflineRender::new(mixer, RATE);

    engine
      .play(AudioPlay {
        gain,
        ..AudioPlay::new(looped(&sound), AudioBus::World)
      })
      .expect("a voice starts");
    engine
      .set_bus_gain(AudioBus::World, gain)
      .expect("the bus takes its gain");
    assert!(render.render(480).iter().all(|sample| *sample == 0.0), "gain {gain}");
  }
}
