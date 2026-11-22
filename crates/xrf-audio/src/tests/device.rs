use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use rtrb::RingBuffer;
use xrf_audio_scene::AudioBus;

use crate::control::audio_play::AudioPlay;
use crate::control::engine::audio_engine_parts::AudioEngineParts;
use crate::decode::decoded_sound::DecodedSound;
use crate::decode::sound_channels::SoundChannels;
use crate::mixer::audio_mixer::AudioMixer;
use crate::output::device::audio_output::AudioOutput;
use crate::output::device::audio_output_choice::AudioOutputChoice;
use crate::output::device::audio_output_status::AudioOutputStatus;
use crate::output::device::stream::mixer_holder::MixerHolder;
use crate::output::device::stream::output_mixer::OutputMixer;
use crate::output::device::stream::output_thread::OutputThread;
use crate::output::offline::offline_render::OfflineRender;
use crate::tests::fixtures::{RATE, engine, looped, once, sine};
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::playback::audio_voice_state::AudioVoiceState;

#[test]
fn hands_the_mixer_back_with_its_voices_when_a_stream_drops_its_callback() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(looped(&sound), AudioBus::Ambience))
    .expect("a voice starts");
  let (returned, mut back) = RingBuffer::new(1);
  let mut holder: MixerHolder = MixerHolder::new(mixer, returned);
  let mut out: Vec<f32> = vec![0.0; 512 * 2];

  holder.render(&mut out, 2, Duration::ZERO);
  drop(holder);

  let mixer: AudioMixer = back.pop().expect("the dropped callback hands the mixer back");
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);

  render.render(512);
  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Playing);
  assert_eq!(engine.get_playback(voice).frame, 1024);
}

// A stream that fails to open, or a callback slow to let go, hands the mixer back past the wait: it is taken up on a
// later look, so audio plays again rather than staying silent for the session.
#[test]
fn takes_up_a_mixer_a_device_hands_back_past_the_wait() {
  let AudioEngineParts { mixer, .. } = engine(4);
  let mut output: OutputMixer = OutputMixer::new(mixer);
  let lent: AudioMixer = output.lend().expect("the thread holds the mixer");
  let (mut handing, returning) = RingBuffer::new(1);

  assert!(output.lend().is_none(), "lent once");
  assert!(!output.take_back(Err(returning)), "the device keeps it past the wait");
  assert!(!output.take_up_late(), "nothing handed back yet");
  assert!(output.get_mut().is_none());

  assert!(handing.push(lent).is_ok());

  assert!(output.take_up_late(), "taken up however late it came");
  assert!(output.get_mut().is_some());
  assert!(!output.take_up_late(), "taken once");
  assert!(output.lend().is_some(), "lent to the next stream");
}

#[test]
fn plays_the_mixer_to_nothing_while_no_device_plays_it() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, 480, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mut mixer } = engine(4);
  let (_requests, received) = mpsc::channel();
  let started: Instant = Instant::now();

  mixer.set_output_rate(RATE);

  let meter = mixer.get_meter().clone();
  let mut output_thread: OutputThread = OutputThread::new(
    mixer,
    AudioOutputChoice::SystemDefault,
    received,
    Arc::new(Mutex::new(AudioOutputStatus::default())),
  );
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(once(&sound), AudioBus::Ambience))
    .expect("a voice starts");

  output_thread.render_silent();
  std::thread::sleep(Duration::from_millis(40));
  output_thread.render_silent();
  engine.collect();

  let elapsed: Duration = started.elapsed();
  let frames: u64 = meter.get_frames();

  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Ended);
  assert_eq!(engine.get_playing_count(), 0, "the ended voice's slot came back");
  assert!(
    frames >= 40 * 48 && frames <= (elapsed.as_secs_f64() * f64::from(RATE)) as u64 + 480,
    "{frames} frames over {elapsed:?}"
  );
}

// The machine may have a device or none; either way the output answers with a status.
#[test]
fn answers_a_missing_device_with_the_default_or_a_failure() {
  let AudioEngineParts { mixer, .. } = engine(4);
  let output: AudioOutput = AudioOutput::start(
    mixer,
    AudioOutputChoice::Device {
      id: String::from("no such device"),
    },
  )
  .expect("the output thread starts");
  let deadline: Instant = Instant::now() + Duration::from_secs(5);
  let mut status: AudioOutputStatus = output.get_status();

  while status.device_name.is_none() && status.failure.is_none() && Instant::now() < deadline {
    std::thread::sleep(Duration::from_millis(10));
    status = output.get_status();
  }

  assert!(
    status.is_fallback || status.failure.is_some(),
    "plays the default in the missing device's place, or says why none plays: {status:?}"
  );

  drop(output);
}
