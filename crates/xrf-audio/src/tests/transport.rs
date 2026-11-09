use std::time::{Duration, Instant};

use crate::tests::fixtures::{RATE, engine, ramp, wav};
use crate::{
  AudioBus, AudioEngine, AudioEngineParts, AudioTransport, AudioTransportPlayback, AudioTransportState, OfflineRender,
};

/// A transport over a mono ramp of `frames` at the test rate, so a rendered sample names the frame it came from.
fn transport(frames: usize) -> AudioTransport {
  AudioTransport::open(wav(RATE, 1, &ramp(frames)), AudioBus::Previews).expect("the sound opens")
}

/// Renders until the output sounds, answering the first frame heard, or `None` after two seconds.
fn first_heard(render: &mut OfflineRender) -> Option<i64> {
  let deadline: Instant = Instant::now() + Duration::from_secs(2);

  while Instant::now() < deadline {
    let out: Vec<f32> = render.render(256);

    if let Some(sample) = out.iter().step_by(2).find(|sample| **sample != 0.0) {
      return Some((sample * 32768.0).round() as i64);
    }

    std::thread::sleep(Duration::from_millis(1));
  }

  None
}

fn state(transport: &mut AudioTransport, engine: &AudioEngine) -> AudioTransportState {
  transport.describe(engine)
}

#[test]
fn opens_stopped_at_the_start_with_the_sound_s_length() {
  let AudioEngineParts { engine, .. } = engine(2);
  let mut transport: AudioTransport = transport(RATE as usize * 2);

  assert_eq!(
    state(&mut transport, &engine),
    AudioTransportState {
      playback: AudioTransportPlayback::Stopped,
      position: Duration::ZERO,
      duration: Duration::from_secs(2),
      is_looped: false,
      failure: None,
    }
  );
}

#[test]
fn refuses_to_open_what_is_not_a_sound() {
  assert!(AudioTransport::open(std::sync::Arc::from(&b"no sound"[..]), AudioBus::Previews).is_err());
}

#[test]
fn plays_pauses_where_it_was_and_resumes_from_there() {
  let AudioEngineParts { mut engine, mixer } = engine(2);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut transport: AudioTransport = transport(20_000);

  transport.play(&mut engine).expect("the transport plays");
  assert_eq!(first_heard(&mut render), Some(1));
  render.render(1000);
  transport.pause(&mut engine).expect("the transport pauses");

  let paused: AudioTransportState = state(&mut transport, &engine);

  assert_eq!(paused.playback, AudioTransportPlayback::Paused);
  render.render(1000);
  assert_eq!(state(&mut transport, &engine).position, paused.position);

  transport.play(&mut engine).expect("the transport plays again");

  let resumed_at: i64 = first_heard(&mut render).expect("the sound resumes");

  assert_eq!(
    resumed_at,
    (paused.position.as_secs_f64() * f64::from(RATE)).round() as i64
  );
}

#[test]
fn seeks_while_playing_and_while_held() {
  let AudioEngineParts { mut engine, mixer } = engine(2);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut transport: AudioTransport = transport(20_000);

  transport
    .seek(&mut engine, Duration::from_millis(100))
    .expect("a held transport seeks");
  assert_eq!(state(&mut transport, &engine).playback, AudioTransportPlayback::Paused);
  assert_eq!(state(&mut transport, &engine).position, Duration::from_millis(100));

  transport.play(&mut engine).expect("the transport plays");
  assert_eq!(first_heard(&mut render), Some(4800));

  transport
    .seek(&mut engine, Duration::from_millis(300))
    .expect("a playing transport seeks");

  // The old voice fades out as the new one's stream fills; once the new voice moves on from 14 400, each block plays
  // on from the frame the transport reports.
  let deadline: Instant = Instant::now() + Duration::from_secs(2);

  while get_frame(state(&mut transport, &engine).position) <= 14_400 && Instant::now() < deadline {
    render.render(256);
    std::thread::sleep(Duration::from_millis(1));
  }

  let at: i64 = get_frame(state(&mut transport, &engine).position);
  let out: Vec<f32> = render.render(256);

  assert!(at > 14_400 && at < 16_000, "{at}");
  assert_eq!((out[0] * 32768.0).round() as i64, at);
  assert_eq!(state(&mut transport, &engine).playback, AudioTransportPlayback::Playing);
}

#[test]
fn ends_at_the_sound_s_end_and_plays_again_from_the_start() {
  let AudioEngineParts { mut engine, mixer } = engine(2);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut transport: AudioTransport = transport(2000);

  transport
    .seek(&mut engine, Duration::from_millis(30))
    .expect("the transport seeks");
  transport.play(&mut engine).expect("the transport plays");
  assert_eq!(first_heard(&mut render), Some(1440));

  let deadline: Instant = Instant::now() + Duration::from_secs(2);

  while state(&mut transport, &engine).playback == AudioTransportPlayback::Playing && Instant::now() < deadline {
    render.render(256);
  }

  let ended: AudioTransportState = state(&mut transport, &engine);

  assert_eq!(ended.playback, AudioTransportPlayback::Ended);
  assert_eq!(ended.position, ended.duration);

  transport.play(&mut engine).expect("the transport plays again");
  assert_eq!(first_heard(&mut render), Some(1));
}

#[test]
fn loops_while_asked_and_stops_back_at_the_start() {
  let AudioEngineParts { mut engine, mixer } = engine(2);
  let mut render: OfflineRender = OfflineRender::new(mixer, RATE);
  let mut transport: AudioTransport = transport(1000);

  transport.set_looped(true);
  transport.play(&mut engine).expect("the transport plays");
  first_heard(&mut render).expect("the sound plays");

  for _ in 0..40 {
    render.render(256);
    std::thread::sleep(Duration::from_millis(1));
  }

  let looping: AudioTransportState = state(&mut transport, &engine);

  assert_eq!(looping.playback, AudioTransportPlayback::Playing);
  assert!(looping.is_looped);

  transport.stop(&mut engine).expect("the transport stops");
  render.render(512);

  let stopped: AudioTransportState = state(&mut transport, &engine);

  assert_eq!(stopped.playback, AudioTransportPlayback::Stopped);
  assert_eq!(stopped.position, Duration::ZERO);
  assert!(render.render(256).iter().all(|sample| *sample == 0.0));
}

fn get_frame(position: Duration) -> i64 {
  (position.as_secs_f64() * f64::from(RATE)).round() as i64
}
