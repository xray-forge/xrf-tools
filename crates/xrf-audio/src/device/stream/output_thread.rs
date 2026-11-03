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
pub(crate) struct OutputThread {
  /// The mixer while no stream plays it.
  mixer: Option<AudioMixer>,
  stream: Option<OutputStream>,
  choice: AudioOutputChoice,
  requests: Receiver<OutputRequest>,
  status: Arc<Mutex<AudioOutputStatus>>,
  /// When to try opening a device again after a failure.
  retry_at: Instant,
}

impl OutputThread {
  /// How often the thread looks at the devices and the stream's errors.
  const CHECK_INTERVAL: Duration = Duration::from_millis(500);

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
    }
  }

  pub(crate) fn run(mut self) {
    loop {
      match self.requests.recv_timeout(Self::CHECK_INTERVAL) {
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
    }
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
