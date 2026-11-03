use std::sync::Arc;
use std::time::{Duration, Instant};

use assert_no_alloc::{AllocDisabler, assert_no_alloc, reset_violation_count, violation_count};

use crate::tests::fixtures::{RATE, engine, play_voices, ramp, sine, wav};
use crate::{
  AudioBus, AudioEngineParts, AudioPlay, AudioVoiceId, DecodedSound, OfflineRender, SoundChannels, SoundStream,
  SoundStreamStart, VoiceSource,
};

/// Counts, rather than forbids, what the audio thread's code allocates or frees inside `assert_no_alloc`.
#[global_allocator]
static ALLOCATOR: AllocDisabler = AllocDisabler;

#[test]
fn counts_an_allocation_inside_the_guard() {
  reset_violation_count();
  assert_no_alloc(|| drop(std::hint::black_box(Vec::<u8>::with_capacity(16))));
  assert_eq!(violation_count(), 2, "one allocation and its free");
}

#[test]
fn renders_starts_stops_and_retires_voices_without_allocating() {
  let sound: Arc<DecodedSound> = Arc::new(sine(22_050, 22_050, 440.0, 0.5, SoundChannels::Mono));
  let short: Arc<DecodedSound> = Arc::new(sine(RATE, 300, 440.0, 0.5, SoundChannels::Stereo));
  let AudioEngineParts { mut engine, mixer } = engine(1056);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut out: Vec<f32> = vec![0.0; 480 * 2];
  let voices: Vec<AudioVoiceId> = play_voices(&mut engine, &sound, 1024);

  for index in 0..16 {
    engine
      .play(AudioPlay::new(
        VoiceSource::Decoded {
          sound: short.clone(),
          start_frame: 0,
          is_looped: false,
        },
        AudioBus::World,
      ))
      .expect("a voice starts");
    engine.pause(voices[index]).expect("a voice pauses");
    engine.stop(voices[index + 16]).expect("a voice stops");
    engine.set_voice_gain(voices[index + 32], 0.5);
    engine.set_voice_rate(voices[index + 48], 1.5);
  }

  engine
    .set_bus_gain(AudioBus::Ambience, 0.5)
    .expect("the bus takes its gain");
  reset_violation_count();

  for _ in 0..100 {
    assert_no_alloc(|| render.render_into(&mut out));
  }

  assert_eq!(violation_count(), 0);
  engine.collect();
}

#[test]
fn streams_without_allocating_on_the_audio_thread() {
  let bytes: Arc<[u8]> = wav(RATE, 1, &ramp(20_000));
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut out: Vec<f32> = vec![0.0; 480 * 2];
  let reader = SoundStream::start(bytes, SoundStreamStart::default()).expect("the stream starts");
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(VoiceSource::Streamed(reader), AudioBus::Previews))
    .expect("a voice starts");
  let deadline: Instant = Instant::now() + Duration::from_secs(5);

  reset_violation_count();

  while engine.get_playback(voice).state != crate::AudioVoiceState::Ended && Instant::now() < deadline {
    assert_no_alloc(|| render.render_into(&mut out));
    std::thread::sleep(Duration::from_micros(200));
  }

  assert_eq!(engine.get_playback(voice).state, crate::AudioVoiceState::Ended);
  assert_eq!(violation_count(), 0);
}
