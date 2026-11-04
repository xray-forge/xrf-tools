use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::HostTrait;
use cpal::{Device, ErrorKind, Host};

use crate::device::audio_devices::AudioDevices;
use crate::device::audio_output_choice::AudioOutputChoice;
use crate::device::audio_output_status::AudioOutputStatus;
use crate::device::stream::open_failure::OpenFailure;
use crate::device::stream::output_request::OutputRequest;
use crate::device::stream::output_stream::OutputStream;
use crate::mixer::audio_mixer::AudioMixer;

/// The thread that owns the device stream: it opens one for the chosen device, follows the system default, and opens
/// another when a device goes, keeping the same mixer and so the same voices throughout.
///
/// While no device plays, the thread renders the mixer itself at the pace a device would and drops what it renders,
/// so commands are taken, voices advance and end, their slots come back, and the mixer's clock runs on.
pub(crate) struct OutputThread {
  /// The mixer while no stream plays it.
  mixer: Option<AudioMixer>,
  stream: Option<OutputStream>,
  choice: AudioOutputChoice,
  requests: Receiver<OutputRequest>,
  status: Arc<Mutex<AudioOutputStatus>>,
  /// When to try opening a device again after a failure.
  retry_at: Instant,
  /// Where the mixer renders while no device plays it.
  silent_buffer: Box<[f32]>,
  /// When the mixer was last rendered without a device, or `None` while a device plays it.
  silent_since: Option<Instant>,
}

impl OutputThread {
  /// How often the thread looks at the devices and the stream's errors.
  const CHECK_INTERVAL: Duration = Duration::from_millis(500);

  /// How often the thread renders the mixer while no device plays it.
  const SILENT_INTERVAL: Duration = Duration::from_millis(10);

  /// The most time one silent render catches up on, so a thread held up for long does not render seconds at once.
  const SILENT_MOST: Duration = Duration::from_millis(250);

  /// How far the frames rendered with no device may drift from the clock's anchor before it moves.
  const MOST_DRIFT: Duration = Duration::from_millis(5);

  /// Frames rendered at a time while no device plays.
  const SILENT_FRAMES: usize = AudioMixer::BLOCK_FRAMES * 4;

  /// How long to wait before trying a device again after it failed to open.
  const RETRY_INTERVAL: Duration = Duration::from_secs(2);

  /// Whether the thread lists the devices while a stream plays, to follow a new default or a chosen device coming
  /// back. Listing CoreAudio's devices during playback is reported to glitch it, so macOS relies on the stream's own
  /// rerouting and device-loss errors alone.
  const IS_POLLING_DEVICES: bool = !cfg!(target_os = "macos");

  pub(crate) fn new(
    mixer: AudioMixer,
    choice: AudioOutputChoice,
    requests: Receiver<OutputRequest>,
    status: Arc<Mutex<AudioOutputStatus>>,
  ) -> Self {
    Self {
      mixer: Some(mixer),
      stream: None,
      choice,
      requests,
      status,
      retry_at: Instant::now(),
      silent_buffer: vec![0.0; Self::SILENT_FRAMES * 2].into_boxed_slice(),
      silent_since: None,
    }
  }

  pub(crate) fn run(mut self) {
    loop {
      let wait: Duration = if self.stream.is_some() {
        Self::CHECK_INTERVAL
      } else {
        Self::SILENT_INTERVAL
      };

      match self.requests.recv_timeout(wait) {
        Ok(OutputRequest::Choose(choice)) => {
          self.choice = choice;
          self.close_stream();
          self.retry_at = Instant::now();
        }
        Ok(OutputRequest::Close) | Err(RecvTimeoutError::Disconnected) => {
          self.close_stream();
          return;
        }
        Err(RecvTimeoutError::Timeout) => {}
      }

      self.check_stream();

      if self.stream.is_none() && Instant::now() >= self.retry_at {
        self.open_stream();
      }

      self.render_silent();
    }
  }

  /// Renders the frames a device would have taken since the last silent render, while the thread holds the mixer.
  pub(crate) fn render_silent(&mut self) {
    let Some(mixer) = &mut self.mixer else {
      self.silent_since = None;
      return;
    };
    let now: Instant = Instant::now();
    // A thread held up past the most it catches up on starts counting again from that far back.
    let since: Instant = self
      .silent_since
      .filter(|since| now.duration_since(*since) <= Self::SILENT_MOST)
      .unwrap_or_else(|| now.checked_sub(Self::SILENT_INTERVAL).unwrap_or(now));
    let rate: f64 = f64::from(mixer.get_output_rate());
    let frames: usize = (now.duration_since(since).as_secs_f64() * rate) as usize;
    let mut left: usize = frames;

    // The frames rendered now would have played from `since` on, with no latency.
    mixer.get_meter().follow_anchor(since, Self::MOST_DRIFT);

    while left > 0 {
      let count: usize = left.min(Self::SILENT_FRAMES);

      mixer.render(&mut self.silent_buffer[..count * 2], 2);
      left -= count;
    }

    // Counting on from the frames rendered, not from now, keeps the part of a frame left over for the next render.
    self.silent_since = Some(since + Duration::from_secs_f64(frames as f64 / rate));
  }

  /// Closes the stream when its device went, or when it no longer plays on the device the choice names.
  fn check_stream(&mut self) {
    let Some(stream) = &mut self.stream else {
      return;
    };
    let mut is_lost: bool = false;

    for kind in stream.take_errors() {
      match kind {
        ErrorKind::DeviceNotAvailable | ErrorKind::StreamInvalidated | ErrorKind::HostUnavailable => {
          log::info!("Audio device stream ended ({kind:?}), reopening");
          is_lost = true;
        }
        // The host rerouted a default-following stream itself.
        ErrorKind::DeviceChanged => stream.device_id = Self::get_default_id(&cpal::default_host()),
        _ => log::debug!("Audio device reported {kind:?}"),
      }
    }

    if !Self::IS_POLLING_DEVICES {
      if is_lost {
        self.close_stream();
        self.retry_at = Instant::now();
      }

      return;
    }

    let host: Host = cpal::default_host();
    let wanted_id: Option<String> = match &self.choice {
      AudioOutputChoice::SystemDefault => Self::get_default_id(&host),
      AudioOutputChoice::Device { id } if AudioDevices::find(&host, id).is_some() => Some(id.clone()),
      AudioOutputChoice::Device { .. } => Self::get_default_id(&host),
    };

    if is_lost || (wanted_id.is_some() && wanted_id != stream.device_id) {
      self.close_stream();
      self.retry_at = Instant::now();
    }
  }

  fn open_stream(&mut self) {
    let Some(mixer) = self.mixer.take() else {
      return;
    };
    let host: Host = cpal::default_host();
    let chosen: Option<Device> = match &self.choice {
      AudioOutputChoice::SystemDefault => None,
      AudioOutputChoice::Device { id } => AudioDevices::find(&host, id),
    };
    let is_fallback: bool = matches!(self.choice, AudioOutputChoice::Device { .. }) && chosen.is_none();
    let Some(device) = chosen.or_else(|| host.default_output_device()) else {
      self.mixer = Some(mixer);
      self.fail(String::from("No output device"));
      return;
    };
    let device_id: Option<String> = AudioDevices::get_id(&device);

    match OutputStream::open(&device, device_id, mixer) {
      Ok(stream) => {
        log::info!(
          "Audio plays on {} at {} Hz, {} channels",
          AudioDevices::get_name(&device),
          stream.config.sample_rate,
          stream.config.channels
        );
        self.set_status(AudioOutputStatus {
          device_name: Some(AudioDevices::get_name(&device)),
          sample_rate: stream.config.sample_rate,
          channels: stream.config.channels,
          is_fallback,
          failure: None,
        });
        self.stream = Some(stream);
      }
      Err(failure) => {
        let OpenFailure { mixer, message } = *failure;

        self.mixer = mixer;
        self.fail(message);
      }
    }
  }

  fn close_stream(&mut self) {
    if let Some(stream) = self.stream.take() {
      self.mixer = stream.close();

      if self.mixer.is_none() {
        log::error!("The audio device kept the mixer after its stream closed; audio stays silent");
      }
    }
  }

  fn fail(&mut self, message: String) {
    let is_new: bool = self
      .status
      .lock()
      .map_or(true, |status| status.failure.as_ref() != Some(&message));

    if is_new {
      log::warn!("No audio output: {message}");
    }

    self.set_status(AudioOutputStatus {
      failure: Some(message),
      ..AudioOutputStatus::default()
    });
    self.retry_at = Instant::now() + Self::RETRY_INTERVAL;
  }

  fn set_status(&self, status: AudioOutputStatus) {
    *self.status.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = status;
  }

  fn get_default_id(host: &Host) -> Option<String> {
    host.default_output_device().as_ref().and_then(AudioDevices::get_id)
  }
}
