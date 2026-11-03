use std::sync::{Mutex, OnceLock, PoisonError};
use std::thread::JoinHandle;

use crate::dlss::Dlss;

/// DLSS starting on a device, as a task whose result is waited for. NGX takes about a second to start, so it starts on
/// a thread of its own while the renderer builds everything else, and whoever asks first waits for what remains.
/// Dropped, it waits for the start to end, so NGX never outlives the renderer that started it.
pub struct DlssTask {
  /// What the start came to, once someone asked.
  started: OnceLock<Option<Dlss>>,
  /// The thread starting it, until someone asks.
  start: Mutex<Option<JoinHandle<Option<Dlss>>>>,
  is_launched: bool,
}

impl DlssTask {
  /// No DLSS: the build, the backend or the GPU has none.
  pub fn none() -> Self {
    Self {
      started: OnceLock::from(None),
      start: Mutex::new(None),
      is_launched: false,
    }
  }

  /// Runs `start` on a thread of its own; none where no thread starts.
  #[cfg(feature = "sdk")]
  pub(crate) fn launch(start: impl FnOnce() -> Option<Dlss> + Send + 'static) -> Self {
    let started: std::io::Result<JoinHandle<Option<Dlss>>> =
      std::thread::Builder::new().name("xrf-dlss".into()).spawn(move || {
        let starting: std::time::Instant = std::time::Instant::now();
        let dlss: Option<Dlss> = start();

        if dlss.is_some() {
          log::info!("DLSS started in {} ms", starting.elapsed().as_millis());
        }

        dlss
      });

    match started {
      Ok(handle) => Self {
        started: OnceLock::new(),
        start: Mutex::new(Some(handle)),
        is_launched: true,
      },
      Err(error) => {
        log::error!("DLSS is unavailable, its thread not starting: {error}");

        Self::none()
      }
    }
  }

  /// Whether DLSS was asked to start, on a GPU and backend it may run on; whether it did is what `get` answers.
  pub fn is_launched(&self) -> bool {
    self.is_launched
  }

  /// DLSS once it started, waiting for the start where it still runs; none where it did not start.
  pub fn get(&self) -> Option<&Dlss> {
    self.started.get_or_init(|| self.join()).as_ref()
  }

  fn join(&self) -> Option<Dlss> {
    let handle: JoinHandle<Option<Dlss>> = self.start.lock().unwrap_or_else(PoisonError::into_inner).take()?;

    handle.join().unwrap_or_else(|_| {
      log::error!("DLSS is unavailable, its start panicked");

      None
    })
  }
}

impl Drop for DlssTask {
  fn drop(&mut self) {
    // Joined, not left running: a start ending after the renderer let its GPU go would leave NGX on a device no one
    // holds.
    let _ = self.get();
  }
}

#[cfg(test)]
mod tests {
  use super::DlssTask;

  #[test]
  fn answers_no_dlss_where_none_was_launched() {
    let starting: DlssTask = DlssTask::none();

    assert!(!starting.is_launched());
    assert!(starting.get().is_none());
  }

  #[cfg(feature = "sdk")]
  #[test]
  fn waits_for_the_start_when_asked_and_when_dropped() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let ended: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let asked: DlssTask = DlssTask::launch({
      let ended: Arc<AtomicBool> = Arc::clone(&ended);

      move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        ended.store(true, Ordering::Release);

        None
      }
    });

    assert!(asked.is_launched());
    assert!(asked.get().is_none(), "a start that found no DLSS answers none");
    assert!(ended.load(Ordering::Acquire), "asking waits for the start");

    let ended: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let dropped: DlssTask = DlssTask::launch({
      let ended: Arc<AtomicBool> = Arc::clone(&ended);

      move || {
        std::thread::sleep(std::time::Duration::from_millis(50));
        ended.store(true, Ordering::Release);

        None
      }
    });

    drop(dropped);

    assert!(ended.load(Ordering::Acquire), "dropping waits for the start");
  }
}
