use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, TryLockError};

use xrf_app_core::TauriResult;
use xrf_app_render::WorldState;
use xrf_renderer::RenderViewportId;
use xrf_utils::format_path;

use crate::viewer::recording::{RecordingOutcome, SlotRecording};

/// The level viewer's single recording slot: it holds the recording while it runs, and after it ends by itself until it
/// is stopped or its page goes. While it holds one, the world hears its viewport.
///
/// Its lock is held while a recording's writer opens, a second or more, so a start, a stop and a release take it on the
/// pool or a thread of their own, never on the async runtime or the event loop. A page that goes is counted apart from
/// it, so a start in flight learns its page went without the release waiting for it.
pub(crate) struct RecordingSlot<R> {
  active: Arc<Mutex<Option<R>>>,
  /// How many times each window's page went: a start that reads the count move finds its page gone.
  releases: Arc<Mutex<HashMap<String, u64>>>,
}

impl<R> Clone for RecordingSlot<R> {
  fn clone(&self) -> Self {
    Self {
      active: Arc::clone(&self.active),
      releases: Arc::clone(&self.releases),
    }
  }
}

impl<R> Default for RecordingSlot<R> {
  fn default() -> Self {
    Self {
      active: Arc::new(Mutex::new(None)),
      releases: Arc::new(Mutex::new(HashMap::new())),
    }
  }
}

impl<R: SlotRecording> RecordingSlot<R> {
  /// Whether a recording is seen running; never waits, so the async runtime may ask. A quick refusal only: a slot
  /// held by a start, a stop or a release reads as free, and [`Self::start`]'s own check decides.
  pub(crate) fn is_recording(&self) -> bool {
    let active = match self.active.try_lock() {
      Ok(active) => active,
      Err(TryLockError::Poisoned(poisoned)) => poisoned.into_inner(),
      Err(TryLockError::WouldBlock) => return false,
    };

    active.as_ref().is_some_and(|recording| !recording.is_ended())
  }

  /// How many times `window`'s page has gone. A start reads it as it is asked and hands it to [`Self::start`].
  pub(crate) fn count_releases(&self, window: &str) -> u64 {
    self.lock_releases().get(window).copied().unwrap_or_default()
  }

  /// Starts the recording `open` opens for the page of `window`, the world then hearing `viewport`, and answers what
  /// `open` answers beside it, such as its file.
  ///
  /// Refused while another runs; one that ended by itself is collected first. `open` runs under the slot's lock, so two
  /// starts never take one free file name. `released` is the count of `window`'s pages gone as the start was asked: one
  /// that moved before `open` refuses it, and one that moved while `open` ran stops what it opened, so no recording
  /// outlives its page.
  ///
  /// # Errors
  ///
  /// Returns an error where a recording runs already, the page went, or `open` fails.
  pub(crate) fn start<T>(
    &self,
    window: &str,
    released: u64,
    world: &WorldState,
    viewport: RenderViewportId,
    open: impl FnOnce() -> TauriResult<(R, T)>,
  ) -> TauriResult<T> {
    // Held from the check until the recording is in place, so two starts asked at once never both pass.
    let mut active: MutexGuard<'_, Option<R>> = self.lock();

    if active.as_ref().is_some_and(|recording| !recording.is_ended()) {
      return Err(String::from("A recording runs already"));
    }

    // Collect a recording that ended by itself (its file already finished) before the next opens its tap.
    if let Some(ended) = active.take() {
      let _ = ended.stop();
    }

    if self.count_releases(window) != released {
      return Err(String::from(
        "The page that asked for the recording went before it started",
      ));
    }

    let (recording, opened): (R, T) = open()?;

    // The page went while the writer opened: its release found the slot held and counted it, so the recording no page
    // would stop ends here.
    if self.count_releases(window) != released {
      drop(active);

      let ended: String = match recording.stop() {
        Ok(outcome) => format!("{} was finished", format_path(&outcome.file)),
        Err(error) => format!("it ended: {error}"),
      };

      return Err(format!(
        "The page that asked for the recording went as it started; {ended}"
      ));
    }

    *active = Some(recording);
    // Pinned under the slot's lock, so a stop waiting on it unpins after, never before.
    world.lock().pin_listener(Some(viewport));

    Ok(opened)
  }

  /// Stops the recording, running or self-ended, and finishes its file; the world hears the focused viewport again.
  ///
  /// # Errors
  ///
  /// Returns an error where the slot holds none, or as [`SlotRecording::stop`] does.
  pub(crate) fn stop(&self, world: &WorldState) -> TauriResult<RecordingOutcome> {
    let recording: R = {
      let mut active: MutexGuard<'_, Option<R>> = self.lock();

      world.lock().pin_listener(None);
      active.take().ok_or("No recording runs")?
    };

    recording.stop()
  }

  /// Stops and finishes the recording `window`'s page started, when that page reloads or the window goes. No page is
  /// left to hear the outcome, so it goes to the log.
  ///
  /// Called on the event loop, so it never waits on the slot: the page is counted gone first, which a start in flight
  /// reads once its recording is open, and a slot another holds is released on a thread of its own.
  pub(crate) fn release_window(&self, window: &str, world: &WorldState) {
    *self.lock_releases().entry(window.to_owned()).or_default() += 1;

    let released: Option<R> = match self.active.try_lock() {
      Ok(mut active) => Self::take_window(&mut active, window, world),
      Err(TryLockError::Poisoned(poisoned)) => Self::take_window(&mut poisoned.into_inner(), window, world),
      Err(TryLockError::WouldBlock) => return self.release_later(window.to_owned(), world.clone()),
    };

    if let Some(recording) = released {
      Self::finish_released(recording, window.to_owned());
    }
  }

  /// The slot, whatever a panicking holder left: a recording is only ever placed whole.
  fn lock(&self) -> MutexGuard<'_, Option<R>> {
    self.active.lock().unwrap_or_else(PoisonError::into_inner)
  }

  fn lock_releases(&self) -> MutexGuard<'_, HashMap<String, u64>> {
    self.releases.lock().unwrap_or_else(PoisonError::into_inner)
  }

  /// The recording `window`'s page started, taken out of the slot with the world's listener let go; none for another
  /// window's.
  fn take_window(active: &mut Option<R>, window: &str, world: &WorldState) -> Option<R> {
    if active
      .as_ref()
      .is_some_and(|recording| recording.get_window() == window)
    {
      world.lock().pin_listener(None);
      active.take()
    } else {
      None
    }
  }

  /// Waits for the slot a start or a stop holds on a thread of its own, then releases `window`'s recording in it.
  fn release_later(&self, window: String, world: WorldState) {
    let slot: Self = self.clone();
    let spawned = std::thread::Builder::new()
      .name(String::from("xrf-recording-release"))
      .spawn(move || {
        let released: Option<R> = Self::take_window(&mut slot.lock(), &window, &world);

        if let Some(recording) = released {
          Self::finish(recording, &window);
        }
      });

    if let Err(error) = spawned {
      log::warn!("The recording slot was not released for a window that went: {error}");
    }
  }

  /// Finishes a released recording on a thread of its own, so its file is closed off the event loop.
  fn finish_released(recording: R, window: String) {
    let spawned = std::thread::Builder::new()
      .name(String::from("xrf-recording-release"))
      .spawn(move || Self::finish(recording, &window));

    if let Err(error) = spawned {
      log::warn!("The recording of a window that went was not finished: {error}");
    }
  }

  fn finish(recording: R, window: &str) {
    match recording.stop() {
      Ok(outcome) => log::info!(
        "The recording of window '{window}' finished as the window went: {}",
        format_path(&outcome.file)
      ),
      Err(error) => log::warn!("The recording of window '{window}' ended as the window went: {error}"),
    }
  }
}
