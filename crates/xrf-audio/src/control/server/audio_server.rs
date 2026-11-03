use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::control::audio_engine::AudioEngine;
use crate::control::audio_engine_parts::AudioEngineParts;
use crate::control::audio_engine_settings::AudioEngineSettings;
use crate::control::server::audio_control::{AudioControl, ControlFlow};
use crate::control::server::audio_event_sink::AudioEventSink;
use crate::control::server::audio_gains::AudioGains;
use crate::control::server::control_request::ControlRequest;
use crate::device::audio_output::AudioOutput;
use crate::device::audio_output_choice::AudioOutputChoice;
use crate::device::audio_output_status::AudioOutputStatus;
use crate::host::audio_asset_source::AudioAssetSource;
use crate::mixer::audio_bus::AudioBus;
use crate::mixer::audio_mixer_meter::AudioMixerMeter;
use crate::scene::audio_scene_inbox::AudioSceneInbox;
use crate::scene::audio_scene_report::AudioSceneReport;
use crate::transport::audio_transport::AudioTransport;
use crate::transport::audio_transport_command::AudioTransportCommand;
use crate::transport::audio_transport_id::AudioTransportId;
use crate::transport::audio_transport_state::AudioTransportState;

/// The audio's control tier: a thread of its own owns the [`AudioEngine`] and every transport, takes requests from any
/// thread, and tells its host what changed through an [`AudioEventSink`]. A clone is a handle on the same thread; the
/// thread ends when the last handle goes or [`Self::shut_down`] runs. The sink must hold no handle, or the thread would
/// keep itself alive.
///
/// Requests that answer (steering or describing a transport, describing a scene) wait for the thread; the rest return
/// at once. A scene's frames go through its own inbox, never waiting. The mixer's clock and load and the device are
/// read without waiting on the thread.
#[derive(Clone)]
pub struct AudioServer {
  shared: Arc<ServerShared>,
}

struct ServerShared {
  requests: SyncSender<ControlRequest>,
  output: Option<AudioOutput>,
  meter: AudioMixerMeter,
  next_transport: AtomicU32,
  thread: Mutex<Option<JoinHandle<()>>>,
}

impl AudioServer {
  /// The longest a playing transport's position goes unsent.
  pub const POSITION_INTERVAL: Duration = Duration::from_millis(50);

  /// How often the control thread looks at its transports and frees the voices the mixer is done with.
  const TICK_INTERVAL: Duration = Duration::from_millis(10);

  /// The longest a request waits for the thread's answer.
  const ANSWER_WAIT: Duration = Duration::from_secs(2);

  /// Requests that queue before a sender waits for the thread.
  const REQUESTS_COUNT: usize = 256;

  /// A server playing on the device `choice` names, with an engine sized by `settings`. A device that cannot start
  /// leaves the server playing to nothing, and the reason in the log.
  ///
  /// # Errors
  ///
  /// Returns an error when the control thread cannot start.
  pub fn start(
    settings: AudioEngineSettings,
    choice: AudioOutputChoice,
    events: impl AudioEventSink,
  ) -> XrfResult<Self> {
    let AudioEngineParts { engine, mixer } = AudioEngineParts::new(settings);
    let output: Option<AudioOutput> = AudioOutput::start(mixer, choice)
      .inspect_err(|error| log::error!("No audio: {error}"))
      .ok();

    Self::start_on(engine, output, events)
  }

  /// A server steering `engine`, whose mixer `output` plays, or a caller renders itself when it is `None`.
  ///
  /// # Errors
  ///
  /// Returns an error when the control thread cannot start.
  pub fn start_on(engine: AudioEngine, output: Option<AudioOutput>, events: impl AudioEventSink) -> XrfResult<Self> {
    let (requests, received) = mpsc::sync_channel(Self::REQUESTS_COUNT);
    let meter: AudioMixerMeter = engine.get_meter().clone();
    let control: AudioControl = AudioControl::new(engine, Box::new(events), Self::POSITION_INTERVAL);
    let thread: JoinHandle<()> = thread::Builder::new()
      .name(String::from("xrf-audio-control"))
      .spawn(move || Self::run(control, &received))
      .map_err(|error| XrfError::new_unexpected_error(format!("Could not start the audio control: {error}")))?;

    Ok(Self {
      shared: Arc::new(ServerShared {
        requests,
        output,
        meter,
        next_transport: AtomicU32::new(1),
        thread: Mutex::new(Some(thread)),
      }),
    })
  }

  /// Moves the master and bus gains to `gains`.
  pub fn set_gains(&self, gains: AudioGains) {
    self.post(ControlRequest::SetGains(gains));
  }

  /// Opens the sound in `bytes` as a transport on `bus`, stopped at its start. The sound is probed on the caller's
  /// thread, so a sound the server cannot play is refused here.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound the server plays, or the control thread has ended.
  pub fn open_transport(&self, bytes: Arc<[u8]>, bus: AudioBus) -> XrfResult<AudioTransportId> {
    let transport: Box<AudioTransport> = Box::new(AudioTransport::open(bytes, bus)?);
    let id: AudioTransportId = AudioTransportId::new(self.shared.next_transport.fetch_add(1, Ordering::Relaxed));

    if self.post(ControlRequest::OpenTransport { id, transport }) {
      Ok(id)
    } else {
      Err(Self::new_ended())
    }
  }

  /// Steers the transport and waits for it to take the command.
  ///
  /// # Errors
  ///
  /// Returns an error for a transport not open, when the transport refuses the command (see [`AudioTransport`]), or
  /// when the control thread does not answer.
  pub fn steer_transport(&self, id: AudioTransportId, command: AudioTransportCommand) -> XrfResult<()> {
    self.ask(|answer| ControlRequest::SteerTransport { id, command, answer })?
  }

  /// Where the transport is now.
  ///
  /// # Errors
  ///
  /// Returns an error for a transport not open, or when the control thread does not answer.
  pub fn describe_transport(&self, id: AudioTransportId) -> XrfResult<AudioTransportState> {
    self
      .ask(|answer| ControlRequest::DescribeTransport { id, answer })?
      .ok_or_else(|| XrfError::new_unexpected_error(format!("No transport {id:?} is open")))
  }

  /// Sends the transport's state on the next tick whether or not it changed, for a watcher that has just come.
  pub fn resend_transport(&self, id: AudioTransportId) {
    self.post(ControlRequest::ResendTransport(id));
  }

  /// Stops and forgets the transport; one already closed is left alone.
  pub fn close_transport(&self, id: AudioTransportId) {
    self.post(ControlRequest::CloseTransport(id));
  }

  /// Opens a scene the control thread plays from the frames sent through the inbox answered, its sounds read through
  /// `assets`. Dropping the inbox closes the scene.
  ///
  /// # Errors
  ///
  /// Returns an error when the control thread has ended.
  pub fn open_scene(&self, assets: Arc<dyn AudioAssetSource>) -> XrfResult<AudioSceneInbox> {
    let (inbox, mailbox) = AudioSceneInbox::open();

    if self.post(ControlRequest::OpenScene { mailbox, assets }) {
      Ok(inbox)
    } else {
      Err(Self::new_ended())
    }
  }

  /// How the scene the host names `scene` plays now, none for a scene not open.
  ///
  /// # Errors
  ///
  /// Returns an error when the control thread does not answer.
  pub fn describe_scene(&self, scene: u32) -> XrfResult<Option<AudioSceneReport>> {
    self.ask(|answer| ControlRequest::DescribeScene { scene, answer })
  }

  /// The device the mixer plays on, or `None` when it could not start.
  pub fn get_output(&self) -> Option<&AudioOutput> {
    self.shared.output.as_ref()
  }

  pub fn get_output_status(&self) -> AudioOutputStatus {
    match &self.shared.output {
      Some(output) => output.get_status(),
      None => AudioOutputStatus {
        failure: Some(String::from("The audio output did not start")),
        ..AudioOutputStatus::default()
      },
    }
  }

  /// The mixer's clock and load.
  pub fn get_meter(&self) -> &AudioMixerMeter {
    &self.shared.meter
  }

  /// Closes every transport and ends the control thread, waiting for it; later requests are refused or dropped.
  pub fn shut_down(&self) {
    self.shared.shut_down();
  }

  fn run(mut control: AudioControl, requests: &Receiver<ControlRequest>) {
    loop {
      match requests.recv_timeout(Self::TICK_INTERVAL) {
        Ok(request) => {
          if control.handle(request) == ControlFlow::ShutDown {
            return;
          }

          while let Ok(request) = requests.try_recv() {
            if control.handle(request) == ControlFlow::ShutDown {
              return;
            }
          }
        }
        Err(RecvTimeoutError::Timeout) => {}
        Err(RecvTimeoutError::Disconnected) => {
          control.close_transports();
          control.close_scenes();
          return;
        }
      }

      control.tick(Instant::now());
    }
  }

  /// Sends `request`, answering whether the thread took it.
  fn post(&self, request: ControlRequest) -> bool {
    self.shared.requests.send(request).is_ok()
  }

  fn ask<T>(&self, request: impl FnOnce(SyncSender<T>) -> ControlRequest) -> XrfResult<T> {
    let (answer, answered) = mpsc::sync_channel(1);

    if !self.post(request(answer)) {
      return Err(Self::new_ended());
    }

    answered.recv_timeout(Self::ANSWER_WAIT).map_err(|error| match error {
      RecvTimeoutError::Timeout => XrfError::new_unexpected_error("The audio control did not answer in time"),
      RecvTimeoutError::Disconnected => Self::new_ended(),
    })
  }

  fn new_ended() -> XrfError {
    XrfError::new_unexpected_error("The audio control has ended")
  }
}

impl ServerShared {
  fn shut_down(&self) {
    let _ = self.requests.send(ControlRequest::ShutDown);

    let thread: Option<JoinHandle<()>> = self.thread.lock().unwrap_or_else(PoisonError::into_inner).take();

    if let Some(thread) = thread {
      let _ = thread.join();
    }
  }
}

impl Drop for ServerShared {
  fn drop(&mut self) {
    self.shut_down();
  }
}
