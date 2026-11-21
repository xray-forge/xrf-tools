use std::sync::Arc;
use std::time::{Duration, Instant};

use assert_no_alloc::{AllocDisabler, assert_no_alloc, reset_violation_count, violation_count};
use xrf_audio_scene::AudioBus;

use crate::control::audio_play::AudioPlay;
use crate::control::engine::audio_engine_parts::AudioEngineParts;
use crate::decode::decoded_sound::DecodedSound;
use crate::decode::sound_channels::SoundChannels;
use crate::mixer::tap::audio_tap::AudioTap;
use crate::output::offline::offline_render::OfflineRender;
use crate::tests::fixtures::{RATE, engine, looped, play_voices, ramp, rms_left, sine, stream, wav};
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::playback::audio_voice_state::AudioVoiceState;
use crate::voice::voice_source::VoiceSource;

// Counts, rather than forbids, what is allocated or freed inside `assert_no_alloc`.
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
          sound: Arc::clone(&short),
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
fn starts_voices_stamped_to_their_frames_without_allocating() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, 4800, 440.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(64);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut out: Vec<f32> = vec![0.0; 480 * 2];

  // Two blocks rendered, so the voices are taken up at the third, at frame 512.
  render.render_into(&mut out);

  for index in 0..32 {
    engine
      .play(AudioPlay {
        // Half a few blocks ahead of the block they are taken up in, and half behind it.
        at: Some(if index % 2 == 0 { 1000 + index * 37 } else { 100 }),
        ..AudioPlay::new(looped(&sound), AudioBus::ALL[index as usize % AudioBus::COUNT])
      })
      .expect("a voice starts");
  }

  reset_violation_count();

  for _ in 0..20 {
    assert_no_alloc(|| render.render_into(&mut out));
  }

  assert_eq!(violation_count(), 0);
  assert!(rms_left(&out) > 0.0);
}

// Filling the tap's ring and letting it go free nothing on the mixer: the tap is freed on the control side.
#[test]
fn taps_the_mix_without_allocating() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, 4800, 440.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(64);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut out: Vec<f32> = vec![0.0; 480 * 2];

  play_voices(&mut engine, &sound, 32);

  let mut tap: AudioTap = engine.open_tap(Duration::from_millis(50)).expect("the tap opens");

  reset_violation_count();

  for _ in 0..20 {
    assert_no_alloc(|| render.render_into(&mut out));
  }

  assert!(tap.read().expect("the stream runs").is_some());
  assert!(tap.get_dropped() > 0, "the ring filled");
  drop(tap);

  for _ in 0..4 {
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
  let reader = stream(bytes, 0, false);
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(VoiceSource::Streamed(reader), AudioBus::Previews))
    .expect("a voice starts");
  let deadline: Instant = Instant::now() + Duration::from_secs(5);

  reset_violation_count();

  while engine.get_playback(voice).state != AudioVoiceState::Ended && Instant::now() < deadline {
    assert_no_alloc(|| render.render_into(&mut out));
    std::thread::sleep(Duration::from_micros(200));
  }

  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Ended);
  assert_eq!(violation_count(), 0);
}
