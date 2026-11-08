use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use crate::tests::fixtures::{RATE, engine, ramp, wav};
use crate::{
  AudioBus, AudioEngineParts, AudioEvent, AudioGains, AudioServer, AudioTransportCommand, AudioTransportId,
  AudioTransportPlayback, AudioTransportState, OfflineRender,
};

/// A server on an engine whose mixer the test renders, and the events it sends.
fn server() -> (AudioServer, OfflineRender, Arc<Mutex<Vec<AudioEvent>>>) {
  let AudioEngineParts { engine, mixer } = engine(4);
  let events: Arc<Mutex<Vec<AudioEvent>>> = Arc::default();
  let sent: Arc<Mutex<Vec<AudioEvent>>> = Arc::clone(&events);
  let server: AudioServer = AudioServer::start_on(engine, None, move |event| {
    sent.lock().unwrap_or_else(PoisonError::into_inner).push(event);
  })
  .expect("the server starts");

  (server, OfflineRender::new(mixer, RATE), events)
}

/// The states sent for `id` so far.
fn sent_states(events: &Mutex<Vec<AudioEvent>>, id: AudioTransportId) -> Vec<AudioTransportState> {
  events
    .lock()
    .unwrap_or_else(PoisonError::into_inner)
    .iter()
    .map(|event| match event {
      AudioEvent::TransportChanged { id: sent, state } => (*sent, state.clone()),
    })
    .filter(|(sent, _)| *sent == id)
    .map(|(_, state)| state)
    .collect()
}

/// Renders until `is_done` holds or two seconds pass, the control thread and the stream's thread running meanwhile.
fn render_until(render: &mut OfflineRender, mut is_done: impl FnMut() -> bool) {
  let deadline: Instant = Instant::now() + Duration::from_secs(2);

  while !is_done() && Instant::now() < deadline {
    render.render(256);
    std::thread::sleep(Duration::from_millis(1));
  }
}

#[test]
fn steers_a_transport_through_the_control_thread_and_pushes_its_state() {
  let (server, mut render, events) = server();
  let id: AudioTransportId = server
    .open_transport(wav(RATE, 1, &ramp(20_000)), AudioBus::Previews)
    .expect("the transport opens");

  server
    .steer_transport(id, AudioTransportCommand::Play)
    .expect("the transport plays");
  render_until(&mut render, || {
    sent_states(&events, id)
      .iter()
      .any(|state| state.position > Duration::ZERO)
  });

  // A tick may push the transport's Stopped state between its opening and the play; from the play on, it plays.
  let states: Vec<AudioTransportState> = sent_states(&events, id)
    .into_iter()
    .skip_while(|state| state.playback == AudioTransportPlayback::Stopped)
    .collect();

  assert!(
    states
      .iter()
      .all(|state| state.playback == AudioTransportPlayback::Playing)
  );
  assert!(states.last().is_some_and(|state| state.position > Duration::ZERO));

  server
    .steer_transport(id, AudioTransportCommand::Seek(Duration::from_millis(100)))
    .expect("the transport seeks");
  server
    .steer_transport(id, AudioTransportCommand::Pause)
    .expect("the transport pauses");

  let paused: AudioTransportState = server.describe_transport(id).expect("the transport is open");

  assert_eq!(paused.playback, AudioTransportPlayback::Paused);
  render_until(&mut render, || {
    sent_states(&events, id)
      .last()
      .is_some_and(|state| state.playback == AudioTransportPlayback::Paused)
  });

  server.close_transport(id);
  assert!(server.describe_transport(id).is_err());
  assert!(server.steer_transport(id, AudioTransportCommand::Play).is_err());
}

#[test]
fn refuses_to_open_what_is_not_a_sound() {
  let (server, _, _) = server();

  assert!(
    server
      .open_transport(Arc::from(&b"no sound"[..]), AudioBus::Previews)
      .is_err()
  );
}

#[test]
fn applies_gains_in_the_order_they_were_sent() {
  let (server, mut render, _) = server();
  let id: AudioTransportId = server
    .open_transport(wav(RATE, 1, &[16_384; 48_000]), AudioBus::Previews)
    .expect("the transport opens");

  server.set_gains(AudioGains {
    master: 0.0,
    ..AudioGains::default()
  });
  server
    .steer_transport(id, AudioTransportCommand::Play)
    .expect("the transport plays");

  // The play was answered, so the gains sent before it reached the mixer's ring before the voice did.
  render_until(&mut render, || {
    server
      .describe_transport(id)
      .is_ok_and(|state| state.position > Duration::from_millis(20))
  });
  assert!(render.render(256).iter().all(|sample| *sample == 0.0));
}

#[test]
fn ends_its_thread_and_closes_its_transports_when_shut_down() {
  let (server, _, _) = server();
  let id: AudioTransportId = server
    .open_transport(wav(RATE, 1, &ramp(1000)), AudioBus::Previews)
    .expect("the transport opens");

  server.shut_down();

  assert!(server.describe_transport(id).is_err());
  assert!(
    server
      .open_transport(wav(RATE, 1, &ramp(1000)), AudioBus::Previews)
      .is_err()
  );
}

#[test]
fn reads_the_mixer_clock_and_load_without_the_thread() {
  let (server, mut render, _) = server();

  render.render(4800);

  assert_eq!(server.get_meter().get_frames(), 4800);
  assert!(server.get_meter().read().load >= 0.0);
  assert_eq!(server.get_meter().read().overruns, 0);
}
