use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use xrf_error::{XrfError, XrfResult};

use crate::device::audio_output_choice::AudioOutputChoice;
use crate::device::audio_output_status::AudioOutputStatus;
use crate::device::stream::output_request::OutputRequest;
use crate::device::stream::output_thread::OutputThread;
use crate::mixer::audio_mixer::AudioMixer;

/// Plays a mixer on an output device from a thread of its own, which follows the system default or the chosen
/// device and opens a new stream when a device goes. No device is not an error: the mixer waits silent and the
/// thread tries again.
pub struct AudioOutput {
  requests: Sender<OutputRequest>,
  status: Arc<Mutex<AudioOutputStatus>>,
  thread: Option<JoinHandle<()>>,
}

impl AudioOutput {
  /// Starts playing `mixer` on the device `choice` names.
  ///
  /// # Errors
  ///
  /// Returns an error when the output thread cannot start.
  pub fn start(mixer: AudioMixer, choice: AudioOutputChoice) -> XrfResult<Self> {
    let (requests, receiver) = mpsc::channel();
    let status: Arc<Mutex<AudioOutputStatus>> = Arc::new(Mutex::new(AudioOutputStatus::default()));
    let output_thread: OutputThread = OutputThread::new(mixer, choice, receiver, status.clone());
    let thread: JoinHandle<()> = thread::Builder::new()
      .name(String::from("xrf-audio-output"))
      .spawn(move || output_thread.run())
      .map_err(|error| XrfError::new_unexpected_error(format!("Could not start the audio output: {error}")))?;

    Ok(Self {
      requests,
      status,
      thread: Some(thread),
    })
  }

  /// Moves playback to the device `choice` names, from the next stream the thread opens.
  pub fn choose(&self, choice: AudioOutputChoice) {
    let _ = self.requests.send(OutputRequest::Choose(choice));
  }

  pub fn get_status(&self) -> AudioOutputStatus {
    self
      .status
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner())
      .clone()
  }
}

impl Drop for AudioOutput {
  fn drop(&mut self) {
    let _ = self.requests.send(OutputRequest::Close);

    if let Some(thread) = self.thread.take() {
      let _ = thread.join();
    }
  }
}
