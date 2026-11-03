use std::time::{Duration, Instant};

use crate::tests::fixtures::{RATE, engine, ramp, wav};
use crate::{
  AudioBus, AudioEngine, AudioEngineParts, AudioPlay, AudioVoiceId, AudioVoiceState, OfflineRender, SoundStream,
  SoundStreamStart, StreamControl, StreamReader, VoiceSource,
};

/// Renders until the voice has sounded, or two seconds pass: the stream's thread fills its ring meanwhile.
fn render_until_audible(engine: &AudioEngine, render: &mut OfflineRender, voice: AudioVoiceId) -> Vec<f32> {
  let deadline: Instant = Instant::now() + Duration::from_secs(2);

  loop {
    let out: Vec<f32> = render.render(256);

    if out.iter().any(|sample| *sample != 0.0) || Instant::now() > deadline {
      return out;
    }

    assert_ne!(engine.get_playback(voice).state, AudioVoiceState::Ended);
    std::thread::sleep(Duration::from_millis(1));
  }
}

fn play(engine: &mut AudioEngine, reader: StreamReader) -> AudioVoiceId {
  engine
    .play(AudioPlay::new(VoiceSource::Streamed(reader), AudioBus::Previews))
    .expect("a voice starts")
}

#[test]
fn starts_a_stream_mid_file_at_the_frame_asked_for() {
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let started: Instant = Instant::now();
  let reader: StreamReader = SoundStream::start(
    wav(RATE, 1, &ramp(20_000)),
    SoundStreamStart {
      frame: 12_345,
      is_looped: false,
    },
  )
  .expect("the stream starts");
  let voice: AudioVoiceId = play(&mut engine, reader);
  let out: Vec<f32> = render_until_audible(&engine, &mut render, voice);

  assert!(
    started.elapsed() < Duration::from_millis(50),
    "{:?} to the first sound",
    started.elapsed()
  );
  assert_eq!((out[0] * 32768.0).round() as i64, 12_345);
  assert_eq!((out[2] * 32768.0).round() as i64, 12_346);
  assert_eq!(out[0], out[1], "a mono stream plays in both channels");
}

#[test]
fn ends_a_stream_at_its_last_frame() {
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let reader: StreamReader = SoundStream::start(
    wav(RATE, 1, &ramp(1000)),
    SoundStreamStart {
      frame: 900,
      is_looped: false,
    },
  )
  .expect("the stream starts");
  let voice: AudioVoiceId = play(&mut engine, reader);
  let out: Vec<f32> = render_until_audible(&engine, &mut render, voice);
  let frames: Vec<i64> = out
    .iter()
    .step_by(2)
    .map(|sample| (sample * 32768.0).round() as i64)
    .collect();

  assert_eq!(&frames[..100], &(900..1000).collect::<Vec<i64>>()[..]);
  assert!(frames[100..].iter().all(|frame| *frame == 0));

  render.render(256);
  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Ended);
}

#[test]
fn loops_a_stream_until_told_otherwise() {
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let reader: StreamReader = SoundStream::start(
    wav(RATE, 1, &ramp(1000)),
    SoundStreamStart {
      frame: 0,
      is_looped: true,
    },
  )
  .expect("the stream starts");
  let control: StreamControl = reader.get_control();
  let voice: AudioVoiceId = play(&mut engine, reader);
  let mut frames: Vec<i64> = Vec::new();
  let deadline: Instant = Instant::now() + Duration::from_secs(2);

  render_until_audible(&engine, &mut render, voice);

  while frames.len() < 3000 && Instant::now() < deadline {
    frames.extend(
      render
        .render(256)
        .iter()
        .step_by(2)
        .map(|sample| (sample * 32768.0).round() as i64),
    );
    std::thread::sleep(Duration::from_millis(1));
  }

  let seams: usize = frames.windows(2).filter(|pair| pair[0] == 999 && pair[1] == 0).count();

  assert!(seams >= 2, "the stream starts over at its end");
  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Playing);

  control.set_looped(false);

  let deadline: Instant = Instant::now() + Duration::from_secs(2);

  while engine.get_playback(voice).state != AudioVoiceState::Ended && Instant::now() < deadline {
    render.render(256);
    std::thread::sleep(Duration::from_millis(1));
  }

  assert_eq!(engine.get_playback(voice).state, AudioVoiceState::Ended);
}

#[test]
fn reports_the_frame_a_stream_plays() {
  let AudioEngineParts { mut engine, mixer } = engine(1);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let reader: StreamReader = SoundStream::start(
    wav(RATE, 1, &ramp(20_000)),
    SoundStreamStart {
      frame: 5000,
      is_looped: false,
    },
  )
  .expect("the stream starts");
  let voice: AudioVoiceId = play(&mut engine, reader);
  let out: Vec<f32> = render_until_audible(&engine, &mut render, voice);
  let first_heard: i64 = (out[0] * 32768.0).round() as i64;
  let reported: u64 = engine.get_playback(voice).frame;

  // The render that first sounded played 256 frames from 5000, so the next to play is 5256.
  assert_eq!(first_heard, 5000);
  assert_eq!(reported, 5256);
}
