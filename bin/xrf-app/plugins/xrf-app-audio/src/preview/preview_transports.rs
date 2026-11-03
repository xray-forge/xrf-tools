use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::thread;
use std::time::Duration;

use tauri::ipc::Channel;
use xrf_app_core::TauriResult;
use xrf_audio::{AudioBus, AudioEngine, AudioTransport, AudioTransportState, SoundPeaks};
use xrf_error::XrfResult;

use crate::engine::AudioEngineState;
use crate::preview::audio_transport_id::AudioTransportId;

/// The previews open in the application: sounds a domain opened for a window's page to play on the Previews bus, as a
/// media element would, each until the page closes it, or its window loads again or goes.
///
/// A watched preview's state goes to its page's channel from a thread of the unit's own whenever it changes, so a
/// player shows its position without asking for it.
#[derive(Clone)]
pub struct PreviewTransports {
  engine: AudioEngineState,
  previews: Arc<Mutex<Previews>>,
}

struct Previews {
  next_id: u32,
  entries: HashMap<AudioTransportId, PreviewEntry>,
  is_shut_down: bool,
}

struct PreviewEntry {
  transport: AudioTransport,
  /// The label of the window whose page opened the preview.
  window: String,
  watcher: Option<Channel<AudioTransportState>>,
  /// What the watcher was last sent, so an unchanged state is not sent again.
  sent: Option<AudioTransportState>,
}

impl PreviewTransports {
  /// How often a watched preview's state is looked at: often enough for a position to move smoothly.
  const WATCH_INTERVAL: Duration = Duration::from_millis(50);

  /// The previews, their watcher thread running until [`Self::shut_down`].
  pub fn start(engine: AudioEngineState) -> Self {
    let previews: Arc<Mutex<Previews>> = Arc::new(Mutex::new(Previews {
      next_id: 1,
      entries: HashMap::new(),
      is_shut_down: false,
    }));
    let watched: Weak<Mutex<Previews>> = Arc::downgrade(&previews);
    let watching: AudioEngineState = engine.clone();

    let started = thread::Builder::new()
      .name(String::from("xrf-audio-previews"))
      .spawn(move || Self::watch_previews(&watched, &watching));

    if let Err(error) = started {
      log::error!("Previews will not report their state: {error}");
    }

    Self { engine, previews }
  }

  /// Opens the sound in `bytes` as a preview for the page of the window labelled `window`, stopped at its start.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a sound the application plays.
  pub fn open(&self, bytes: Arc<[u8]>, window: &str) -> TauriResult<AudioTransportId> {
    let transport: AudioTransport =
      AudioTransport::open(bytes, AudioBus::Previews).map_err(|error| format!("Cannot preview the sound: {error}"))?;
    let mut previews: MutexGuard<'_, Previews> = self.lock();
    let id: AudioTransportId = AudioTransportId::new(previews.next_id);

    previews.next_id = previews.next_id.wrapping_add(1);
    previews.entries.insert(
      id,
      PreviewEntry {
        transport,
        window: window.to_owned(),
        watcher: None,
        sent: None,
      },
    );

    Ok(id)
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open, or when the sound cannot play.
  pub fn play(&self, id: AudioTransportId) -> TauriResult<()> {
    self.steer(id, AudioTransport::play)
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open, or when the audio thread takes no commands.
  pub fn pause(&self, id: AudioTransportId) -> TauriResult<()> {
    self.steer(id, AudioTransport::pause)
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open, or when the audio thread takes no commands.
  pub fn stop(&self, id: AudioTransportId) -> TauriResult<()> {
    self.steer(id, AudioTransport::stop)
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open, or when the sound cannot play from there.
  pub fn seek(&self, id: AudioTransportId, position: Duration) -> TauriResult<()> {
    self.steer(id, |transport, engine| transport.seek(engine, position))
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open.
  pub fn set_looped(&self, id: AudioTransportId, is_looped: bool) -> TauriResult<()> {
    self.steer(id, |transport, _| {
      transport.set_looped(is_looped);
      Ok(())
    })
  }

  /// # Errors
  ///
  /// Returns an error for a preview not open.
  pub fn describe(&self, id: AudioTransportId) -> TauriResult<AudioTransportState> {
    self.steer(id, |transport, engine| Ok(transport.describe(engine)))
  }

  /// Sends the preview's state to `watcher` now and whenever it changes, until the preview closes; a second watcher
  /// takes the first's place.
  ///
  /// # Errors
  ///
  /// Returns an error for a preview not open.
  pub fn watch(&self, id: AudioTransportId, watcher: Channel<AudioTransportState>) -> TauriResult<()> {
    let mut previews: MutexGuard<'_, Previews> = self.lock();
    let entry: &mut PreviewEntry = previews.entries.get_mut(&id).ok_or_else(|| Self::new_missing(id))?;

    entry.watcher = Some(watcher);
    entry.sent = None;
    drop(previews);

    self.send_changes();

    Ok(())
  }

  /// The loudest sample of each of `count` stretches of the preview's sound, decoded on the caller's thread.
  ///
  /// # Errors
  ///
  /// Returns an error for a preview not open, or a sound that fails to decode.
  pub fn read_peaks(&self, id: AudioTransportId, count: usize) -> TauriResult<SoundPeaks> {
    let bytes: Arc<[u8]> = self
      .lock()
      .entries
      .get(&id)
      .map(|entry| entry.transport.get_bytes().clone())
      .ok_or_else(|| Self::new_missing(id))?;

    SoundPeaks::read(bytes, count).map_err(|error| format!("Cannot read the sound's waveform: {error}"))
  }

  /// Stops and forgets the preview; one already closed is left alone.
  pub fn close(&self, id: AudioTransportId) {
    let entry: Option<PreviewEntry> = self.lock().entries.remove(&id);

    if let Some(mut entry) = entry {
      Self::stop_entry(&self.engine, &mut entry);
    }
  }

  /// Stops and forgets the previews the window labelled `window` opened, as its page loads again or the window goes:
  /// the page that could close them is gone, and another window's previews play on.
  pub fn close_window(&self, window: &str) {
    self.close_where(|entry| entry.window == window);
  }

  /// Closes every preview and ends the watcher thread, as the application exits.
  pub fn shut_down(&self) {
    self.close_where(|_| true);
    self.lock().is_shut_down = true;
  }

  fn close_where(&self, is_closed: impl Fn(&PreviewEntry) -> bool) {
    let entries: Vec<PreviewEntry> = self
      .lock()
      .entries
      .extract_if(|_, entry| is_closed(entry))
      .map(|(_, entry)| entry)
      .collect();

    for mut entry in entries {
      Self::stop_entry(&self.engine, &mut entry);
    }
  }

  fn steer<T>(
    &self,
    id: AudioTransportId,
    operation: impl FnOnce(&mut AudioTransport, &mut AudioEngine) -> XrfResult<T>,
  ) -> TauriResult<T> {
    let mut previews: MutexGuard<'_, Previews> = self.lock();
    let entry: &mut PreviewEntry = previews.entries.get_mut(&id).ok_or_else(|| Self::new_missing(id))?;

    self
      .engine
      .with_engine(|engine| operation(&mut entry.transport, engine))
      .map_err(|error| error.to_string())
  }

  /// Sends every watched preview's state that changed since it was last sent.
  fn send_changes(&self) {
    Self::send_changed(&mut self.lock(), &self.engine);
  }

  fn send_changed(previews: &mut Previews, engine: &AudioEngineState) {
    for entry in previews.entries.values_mut() {
      let Some(watcher) = &entry.watcher else {
        continue;
      };
      let state: AudioTransportState = engine.with_engine(|engine| entry.transport.describe(engine));

      if entry.sent.as_ref() == Some(&state) {
        continue;
      }

      // A page that went cannot be told: its channel refuses, and the preview stops being watched.
      if watcher.send(state.clone()).is_err() {
        entry.watcher = None;
      }

      entry.sent = Some(state);
    }
  }

  /// The watcher thread: sends changes until the previews go or shut down.
  fn watch_previews(previews: &Weak<Mutex<Previews>>, engine: &AudioEngineState) {
    loop {
      thread::sleep(Self::WATCH_INTERVAL);

      let Some(previews) = previews.upgrade() else {
        return;
      };
      let mut previews: MutexGuard<'_, Previews> = previews.lock().unwrap_or_else(PoisonError::into_inner);

      if previews.is_shut_down {
        return;
      }

      Self::send_changed(&mut previews, engine);
    }
  }

  fn stop_entry(engine: &AudioEngineState, entry: &mut PreviewEntry) {
    if let Err(error) = engine.with_engine(|engine| entry.transport.stop(engine)) {
      log::warn!("A closed preview may play on: {error}");
    }
  }

  fn lock(&self) -> MutexGuard<'_, Previews> {
    self.previews.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn new_missing(id: AudioTransportId) -> String {
    format!("No preview {id:?} is open")
  }
}
