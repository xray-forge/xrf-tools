use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use xrf_audio::{
  AudioBus, AudioEngine, AudioEngineParts, AudioEngineSettings, AudioOutput, AudioOutputStatus, AudioRefusal,
};

use crate::settings::AudioSettings;

/// The application's one audio engine under its lock, the device it plays on, and the settings it plays with. A unit
/// that plays sound holds a clone.
#[derive(Clone)]
pub struct AudioEngineState {
  inner: Arc<Mutex<EngineInner>>,
}

struct EngineInner {
  engine: AudioEngine,
  /// `None` when the output thread could not start: the engine then plays to nothing.
  output: Option<AudioOutput>,
  settings: AudioSettings,
  /// The windows that have the focus, by label, or `None` before any window said: the application counts as focused
  /// until then, so it never starts muted, and while any window has the focus, whatever order their events come in.
  focused_windows: Option<HashSet<String>>,
}

impl AudioEngineState {
  /// An engine playing on the device `settings` names, at their volumes.
  pub fn new(settings: AudioSettings) -> Self {
    let AudioEngineParts { engine, mixer } = AudioEngineParts::new(AudioEngineSettings::default());
    let output: Option<AudioOutput> = AudioOutput::start(mixer, settings.output.clone())
      .inspect_err(|error| log::error!("No audio: {error}"))
      .ok();
    let mut inner: EngineInner = EngineInner {
      engine,
      output,
      settings,
      focused_windows: None,
    };

    inner.apply_volumes();

    Self {
      inner: Arc::new(Mutex::new(inner)),
    }
  }

  /// Plays with `settings` from now on: their volumes, and their device once the output thread opens it.
  pub fn apply(&self, settings: AudioSettings) {
    let mut inner: MutexGuard<'_, EngineInner> = self.lock();

    if inner.settings.output != settings.output
      && let Some(output) = &inner.output
    {
      output.choose(settings.output.clone());
    }

    inner.settings = settings;
    inner.apply_volumes();
  }

  /// Notes whether the window `label` has the focus: the application mutes in the background, where the settings say
  /// so, once no window has it.
  pub fn set_window_focused(&self, label: &str, is_focused: bool) {
    let mut inner: MutexGuard<'_, EngineInner> = self.lock();
    let was_muted: bool = inner.is_muted();
    let windows: &mut HashSet<String> = inner.focused_windows.get_or_insert_default();

    if is_focused {
      windows.insert(label.to_owned());
    } else {
      windows.remove(label);
    }

    if inner.is_muted() != was_muted {
      inner.apply_volumes();
    }
  }

  /// Whether the application plays silent because it is in the background.
  pub fn is_muted(&self) -> bool {
    self.lock().is_muted()
  }

  pub fn get_output_status(&self) -> AudioOutputStatus {
    match &self.lock().output {
      Some(output) => output.get_status(),
      None => AudioOutputStatus {
        failure: Some(String::from("The audio output did not start")),
        ..AudioOutputStatus::default()
      },
    }
  }

  /// Runs `operation` on the engine under its lock.
  pub fn with_engine<T>(&self, operation: impl FnOnce(&mut AudioEngine) -> T) -> T {
    operation(&mut self.lock().engine)
  }

  fn lock(&self) -> MutexGuard<'_, EngineInner> {
    self.inner.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl EngineInner {
  fn is_muted(&self) -> bool {
    let is_focused: bool = self.focused_windows.as_ref().is_none_or(|windows| !windows.is_empty());

    self.settings.is_muted_in_background && !is_focused
  }

  fn apply_volumes(&mut self) {
    let master: f32 = if self.is_muted() {
      0.0
    } else {
      self.settings.volumes.master
    };
    let mut refusal: Option<AudioRefusal> = self.engine.set_master_gain(master).err();

    for bus in AudioBus::ALL {
      refusal = refusal.or(self.engine.set_bus_gain(bus, self.settings.volumes.get_bus(bus)).err());
    }

    if let Some(refusal) = refusal {
      log::warn!("Audio volumes not applied: {refusal}");
    }
  }
}
