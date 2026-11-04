use std::collections::HashMap;
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::control::audio_engine::AudioEngine;
use crate::control::server::audio_event::AudioEvent;
use crate::control::server::audio_event_sink::AudioEventSink;
use crate::control::server::audio_gains::AudioGains;
use crate::control::server::control_request::ControlRequest;
use crate::control::server::control_scene::ControlScene;
use crate::mixer::audio_bus::AudioBus;
use crate::render::scene::scene_tick::SceneTick;
use crate::render::sound_bank::SoundBank;
use crate::render::stream_launcher::StreamLauncher;
use crate::scene::audio_scene_report::AudioSceneReport;
use crate::transport::audio_transport::AudioTransport;
use crate::transport::audio_transport_command::AudioTransportCommand;
use crate::transport::audio_transport_id::AudioTransportId;
use crate::transport::audio_transport_state::AudioTransportState;

/// What the control thread owns: the engine, the transports and the scenes open on it, steered by requests and looked
/// at once a tick. A plain state machine, so a test or an offline session drives it without the thread.
pub(crate) struct AudioControl {
  engine: AudioEngine,
  transports: HashMap<AudioTransportId, OpenTransport>,
  scenes: Vec<ControlScene>,
  bank: SoundBank,
  streams: StreamLauncher,
  events: Box<dyn AudioEventSink>,
  /// How often a playing transport's position is sent.
  position_interval: Duration,
  /// Whether placed sounds are heard for headphones or speakers.
  spatialization: AudioSpatialization,
}

struct OpenTransport {
  transport: Box<AudioTransport>,
  /// What was last sent of it, and when.
  sent: Option<(AudioTransportState, Instant)>,
}

/// Whether the control thread goes on after a request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ControlFlow {
  Continue,
  ShutDown,
}

impl AudioControl {
  /// A control reading sounds on a thread of its own and streaming each on its own thread.
  pub(crate) fn new(engine: AudioEngine, events: Box<dyn AudioEventSink>, position_interval: Duration) -> Self {
    Self::with(
      engine,
      events,
      position_interval,
      SoundBank::threaded(),
      StreamLauncher::Threads,
    )
  }

  pub(crate) fn with(
    engine: AudioEngine,
    events: Box<dyn AudioEventSink>,
    position_interval: Duration,
    bank: SoundBank,
    streams: StreamLauncher,
  ) -> Self {
    Self {
      engine,
      transports: HashMap::new(),
      scenes: Vec::new(),
      bank,
      streams,
      events,
      position_interval,
      spatialization: AudioSpatialization::default(),
    }
  }

  pub(crate) fn get_streams(&mut self) -> &mut StreamLauncher {
    &mut self.streams
  }

  pub(crate) fn handle(&mut self, request: ControlRequest) -> ControlFlow {
    match request {
      ControlRequest::SetGains(gains) => self.set_gains(gains),
      ControlRequest::OpenTransport { id, transport } => {
        self.transports.insert(id, OpenTransport { transport, sent: None });
      }
      ControlRequest::SteerTransport { id, command, answer } => {
        let _ = answer.send(self.steer(id, command));
      }
      ControlRequest::DescribeTransport { id, answer } => {
        let state: Option<AudioTransportState> = self
          .transports
          .get_mut(&id)
          .map(|open| open.transport.describe(&self.engine));

        let _ = answer.send(state);
      }
      ControlRequest::ResendTransport(id) => {
        if let Some(open) = self.transports.get_mut(&id) {
          open.sent = None;
        }
      }
      ControlRequest::CloseTransport(id) => self.close_transport(id),
      ControlRequest::OpenScene {
        mailbox,
        assets,
        geometry,
      } => {
        let scene: ControlScene = ControlScene::new(mailbox, assets, self.engine.get_acoustics(), geometry);

        self.scenes.push(scene);
      }
      ControlRequest::SetSpatialization(spatialization) => self.spatialization = spatialization,
      ControlRequest::DescribeScene { scene, answer } => {
        let _ = answer.send(self.describe_scene(scene));
      }
      ControlRequest::ShutDown => {
        self.close_transports();
        self.close_scenes();
        return ControlFlow::ShutDown;
      }
    }

    ControlFlow::Continue
  }

  /// Frees the voices the mixer is done with, plays every scene on to the mixer's clock, and sends what changed of the
  /// transports since the last tick.
  pub(crate) fn tick(&mut self, now: Instant) {
    self.engine.collect();
    self.bank.collect();
    self.tick_scenes();

    for (id, open) in &mut self.transports {
      let state: AudioTransportState = open.transport.describe(&self.engine);
      let is_due: bool = match &open.sent {
        None => true,
        Some((sent, sent_at)) => {
          Self::is_changed_besides_position(sent, &state)
            || (sent.position != state.position && now.duration_since(*sent_at) >= self.position_interval)
        }
      };

      if is_due {
        self.events.send(AudioEvent::TransportChanged {
          id: *id,
          state: state.clone(),
        });
        open.sent = Some((state, now));
      }
    }
  }

  /// How the scene the host names `scene` plays as of the last tick, none for a scene not open.
  pub(crate) fn describe_scene(&self, scene: u32) -> Option<AudioSceneReport> {
    let open: &ControlScene = self.scenes.iter().find(|open| open.get_scene() == scene)?;

    Some(AudioSceneReport {
      mixer_load: self.engine.get_meter().get_load(),
      ..open.get_render().report()
    })
  }

  /// Stops and forgets every transport.
  pub(crate) fn close_transports(&mut self) {
    let ids: Vec<AudioTransportId> = self.transports.keys().copied().collect();

    for id in ids {
      self.close_transport(id);
    }
  }

  /// Ends every scene's voices and forgets the scenes.
  pub(crate) fn close_scenes(&mut self) {
    for scene in &mut self.scenes {
      scene.close(&mut self.engine);
    }

    self.scenes.clear();
  }

  fn tick_scenes(&mut self) {
    let (frame, rate): (u64, u32) = (self.engine.get_meter().get_frames(), self.engine.get_meter().get_rate());

    self.scenes.retain_mut(|scene| {
      if scene.is_closed() {
        scene.close(&mut self.engine);

        return false;
      }

      scene.tick(SceneTick {
        engine: &mut self.engine,
        bank: &mut self.bank,
        streams: &mut self.streams,
        spatialization: self.spatialization,
        mixer_frame: frame,
        rate,
      });

      true
    });
  }

  fn set_gains(&mut self, gains: AudioGains) {
    let mut refusal = self.engine.set_master_gain(gains.master).err();

    for bus in AudioBus::ALL {
      refusal = refusal.or(self.engine.set_bus_gain(bus, gains.get_bus(bus)).err());
    }

    if let Some(refusal) = refusal {
      log::warn!("Audio gains not applied: {refusal}");
    }
  }

  fn steer(&mut self, id: AudioTransportId, command: AudioTransportCommand) -> XrfResult<()> {
    let open: &mut OpenTransport = self
      .transports
      .get_mut(&id)
      .ok_or_else(|| XrfError::new_unexpected_error(format!("No transport {id:?} is open")))?;
    let transport: &mut AudioTransport = &mut open.transport;
    let engine: &mut AudioEngine = &mut self.engine;

    match command {
      AudioTransportCommand::Play => transport.play(engine),
      AudioTransportCommand::Pause => transport.pause(engine),
      AudioTransportCommand::Stop => transport.stop(engine),
      AudioTransportCommand::Seek(position) => transport.seek(engine, position),
      AudioTransportCommand::SetLooped(is_looped) => {
        transport.set_looped(is_looped);
        Ok(())
      }
    }
  }

  fn close_transport(&mut self, id: AudioTransportId) {
    if let Some(mut open) = self.transports.remove(&id)
      && let Err(error) = open.transport.stop(&mut self.engine)
    {
      log::warn!("A closed transport may play on: {error}");
    }
  }

  fn is_changed_besides_position(sent: &AudioTransportState, state: &AudioTransportState) -> bool {
    sent.playback != state.playback
      || sent.duration != state.duration
      || sent.is_looped != state.is_looped
      || sent.failure != state.failure
  }
}
