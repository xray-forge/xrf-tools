use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rtrb::RingBuffer;

use crate::device::stream::mixer_holder::MixerHolder;
use crate::device::stream::output_thread::OutputThread;
use crate::tests::fixtures::{RATE, engine, looped, once, sine};
use crate::{
  AudioBus, AudioEngineParts, AudioMixer, AudioOutput, AudioOutputChoice, AudioOutputStatus, AudioPlay, AudioVoiceId,
  AudioVoiceState, DecodedSound, OfflineRender, SoundChannels,
};

/// A device that goes drops its stream's callback: the mixer comes back with its voices, to play on the next stream.
#[test]
fn hands_the_mixer_back_with_its_voices_when_a_stream_drops_its_callback() {
  let sound: Arc<DecodedSound> = Arc::new(sine(RATE, RATE as usize, 1000.0, 0.5, SoundChannels::Mono));
  let AudioEngineParts { mut engine, mixer } = engine(4);
  let voice: AudioVoiceId = engine
    .play(AudioPlay::new(looped(&sound), AudioBus::Ambience))
    .expect("a voice starts");
  let (returned, mut back) = RingBuffer::new(1);
  let mut holder: MixerHolder = MixerHolder::new(mixer, returned);
  let mut out: Vec<f32> = vec![0.0; 480 * 2];

  holder.render(&mut out, 2);
  drop(holder);

  let mixer: AudioMixer = back.pop().expect("the dropped callback hands the mixer back");
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);

  render.render(480);
  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Playing);
  assert_eq!(engine.get_playback(voice).frame, 960);
}

/// While no device plays, the output thread renders the mixer at a device's pace: commands are taken, voices play out
/// and come back to be freed, and the mixer's clock runs on.
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

/// The machine may have a device or none: either way the output answers with a status rather than a panic, and closes.
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
