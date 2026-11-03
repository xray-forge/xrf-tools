use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tauri::ipc::Channel;
use xrf_app_core::TauriResult;
use xrf_audio::{
  AudioBus, AudioEvent, AudioEventSink, AudioServer, AudioTransportCommand, AudioTransportId, AudioTransportState,
  SoundPeaks,
};

/// The previews open in the application: sounds a domain opened on the audio server for a window's page to play on
/// the Previews bus, as a media element would, each until the page closes it, or its window loads again or goes.
///
/// The server's control thread pushes each preview's changed state, its position included; it goes to the channel of
/// the page watching it.
#[derive(Clone)]
pub struct PreviewTransports {
  server: AudioServer,
  previews: PreviewTable,
}

/// The previews by id, shared with the server's event sink.
type PreviewTable = Arc<Mutex<HashMap<AudioTransportId, PreviewEntry>>>;

struct PreviewEntry {
  /// The label of the window whose page opened the preview.
  window: String,
  /// The sound's bytes, for its waveform.
  bytes: Arc<[u8]>,
  watcher: Option<Channel<AudioTransportState>>,
}

/// The server's event sink: hands each preview's state to the page watching it. It holds the table, not the server.
#[derive(Clone, Default)]
pub struct PreviewEvents {
  previews: PreviewTable,
}

impl AudioEventSink for PreviewEvents {
  fn send(&mut self, event: AudioEvent) {
    match event {
      AudioEvent::TransportChanged { id, state } => {
        let mut previews: MutexGuard<'_, HashMap<AudioTransportId, PreviewEntry>> = lock(&self.previews);

        if let Some(entry) = previews.get_mut(&id)
          && let Some(watcher) = &entry.watcher
          // A page that went cannot be told: its channel refuses, and the preview stops being watched.
          && watcher.send(state).is_err()
        {
          entry.watcher = None;
        }
      }
    }
  }
}

impl PreviewTransports {
  /// The previews on `server`, which was started with `events`.
  pub fn new(server: AudioServer, events: &PreviewEvents) -> Self {
    Self {
      server,
      previews: events.previews.clone(),
    }
  }

  /// Opens the sound in `bytes` as a preview for the page of the window labelled `window`, stopped at its start.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound the application plays.
  pub fn open(&self, bytes: Arc<[u8]>, window: &str) -> TauriResult<AudioTransportId> {
    let id: AudioTransportId = self
      .server
      .open_transport(bytes.clone(), AudioBus::Previews)
      .map_err(|error| format!("Cannot preview the sound: {error}"))?;

    lock(&self.previews).insert(
      id,
      PreviewEntry {
        window: window.to_owned(),
        bytes,
        watcher: None,
      },
    );

    Ok(id)
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open, or when the sound cannot play.
  pub fn play(&self, id: AudioTransportId) -> TauriResult<()> {
    self.steer(id, AudioTransportCommand::Play)
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open, or when the audio thread takes no commands.
  pub fn pause(&self, id: AudioTransportId) -> TauriResult<()> {
    self.steer(id, AudioTransportCommand::Pause)
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open, or when the audio thread takes no commands.
  pub fn stop(&self, id: AudioTransportId) -> TauriResult<()> {
    self.steer(id, AudioTransportCommand::Stop)
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open, or when the sound cannot play from there.
  pub fn seek(&self, id: AudioTransportId, position: Duration) -> TauriResult<()> {
    self.steer(id, AudioTransportCommand::Seek(position))
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open.
  pub fn set_looped(&self, id: AudioTransportId, is_looped: bool) -> TauriResult<()> {
    self.steer(id, AudioTransportCommand::SetLooped(is_looped))
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open.
  pub fn describe(&self, id: AudioTransportId) -> TauriResult<AudioTransportState> {
    self.server.describe_transport(id).map_err(|error| error.to_string())
  }

  /// Sends the preview's state to `watcher` from the server's next tick and whenever it changes, until the preview
  /// closes; a second watcher takes the first's place.
  ///
  /// # Errors
  ///
  /// Returns an error for a preview not open.
  pub fn watch(&self, id: AudioTransportId, watcher: Channel<AudioTransportState>) -> TauriResult<()> {
    lock(&self.previews)
      .get_mut(&id)
      .ok_or_else(|| Self::new_missing(id))?
      .watcher = Some(watcher);
    self.server.resend_transport(id);

    Ok(())
  }

  /// The loudest sample of each of `count` stretches of the preview's sound, decoded on the caller's thread.
  ///
  /// # Errors
  ///
  /// Returns an error for a preview not open, or a sound that fails to decode.
  pub fn read_peaks(&self, id: AudioTransportId, count: usize) -> TauriResult<SoundPeaks> {
    let bytes: Arc<[u8]> = lock(&self.previews)
      .get(&id)
      .map(|entry| entry.bytes.clone())
      .ok_or_else(|| Self::new_missing(id))?;

    SoundPeaks::read(bytes, count).map_err(|error| format!("Cannot read the sound's waveform: {error}"))
  }

  /// Stops and forgets the preview; one already closed is left alone.
  pub fn close(&self, id: AudioTransportId) {
    if lock(&self.previews).remove(&id).is_some() {
      self.server.close_transport(id);
    }
  }

  /// Stops and forgets the previews the window labelled `window` opened, as its page loads again or the window goes:
  /// the page that could close them is gone, and another window's previews play on.
  pub fn close_window(&self, window: &str) {
    let closed: Vec<AudioTransportId> = lock(&self.previews)
      .extract_if(|_, entry| entry.window == window)
      .map(|(id, _)| id)
      .collect();

    for id in closed {
      self.server.close_transport(id);
    }
  }

  /// Closes every preview and ends the server's control thread, as the application exits.
  pub fn shut_down(&self) {
    self.server.shut_down();
    lock(&self.previews).clear();
  }

  fn steer(&self, id: AudioTransportId, command: AudioTransportCommand) -> TauriResult<()> {
    if !lock(&self.previews).contains_key(&id) {
      return Err(Self::new_missing(id));
    }

    self
      .server
      .steer_transport(id, command)
      .map_err(|error| error.to_string())
  }

  fn new_missing(id: AudioTransportId) -> String {
    format!("No preview {id:?} is open")
  }
}

fn lock(previews: &PreviewTable) -> MutexGuard<'_, HashMap<AudioTransportId, PreviewEntry>> {
  previews.lock().unwrap_or_else(PoisonError::into_inner)
}
