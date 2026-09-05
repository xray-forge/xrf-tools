use std::sync::{Arc, Mutex, OnceLock};

use xrf_error::XrfResult;

use crate::ltx::Ltx;

/// One root's resolved value, and the right to produce it.
///
/// A [`OnceLock`] alone deduplicates the value but not the work: every thread arriving before the cell is set runs the
/// whole resolution and all but one throw the result away. For a config root that is its entire include graph read and
/// lowered again per racing thread - a `gamedata verify` sweep over a vanilla tree made 374 extra document reads at 32
/// workers against 1 - and it left the cache's own hit count depending on the schedule.
#[derive(Debug, Default)]
pub(crate) struct LtxResolvedRoot {
  value: OnceLock<Arc<Ltx>>,
  /// Held only while a value is being produced, so racing threads wait for the first rather than repeating it.
  ///
  /// Guards no state of its own, so a panic inside a resolution leaves nothing inconsistent and the poison is stepped
  /// over rather than failing every later caller.
  gate: Mutex<()>,
}

impl LtxResolvedRoot {
  /// The resolved value, produced once however many threads ask at the same time.
  ///
  /// # Errors
  ///
  /// Whatever `produce` answers. A failure is not remembered: the next caller tries again, so a transient read error
  /// does not poison a root for the life of the project.
  pub(crate) fn get_or_try_init<F>(&self, produce: F) -> XrfResult<Arc<Ltx>>
  where
    F: FnOnce() -> XrfResult<Arc<Ltx>>,
  {
    if let Some(resolved) = self.value.get() {
      return Ok(Arc::clone(resolved));
    }

    let _guard = self.gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    // Whoever held the gate before this thread may have published, which is what makes this one flight rather than a
    // queue of threads each repeating the same resolution.
    if let Some(resolved) = self.value.get() {
      return Ok(Arc::clone(resolved));
    }

    // `?` before the cell is touched, so a failure stores nothing and the next caller is free to try again.
    let resolved: Arc<Ltx> = produce()?;

    Ok(Arc::clone(self.value.get_or_init(|| resolved)))
  }
}

#[cfg(test)]
mod test {
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::sync::{Arc, Barrier};
  use std::thread;

  use xrf_error::{XrfError, XrfResult};

  use crate::ltx::Ltx;
  use crate::project::LtxResolvedRoot;

  #[test]
  fn produces_once_however_many_threads_ask_at_the_same_time() {
    let root: Arc<LtxResolvedRoot> = Arc::new(LtxResolvedRoot::default());
    let produced: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
    // Every thread is inside `get_or_try_init` before any of them may finish, which is the race a `OnceLock` alone
    // loses: without the gate each of them runs the whole resolution.
    let barrier: Arc<Barrier> = Arc::new(Barrier::new(8));

    let workers: Vec<_> = (0..8)
      .map(|_| {
        let (root, produced, barrier) = (Arc::clone(&root), Arc::clone(&produced), Arc::clone(&barrier));

        thread::spawn(move || {
          barrier.wait();

          root
            .get_or_try_init(|| {
              produced.fetch_add(1, Ordering::SeqCst);

              Ok(Arc::new(Ltx::read_from_str("[section]\nkey = value\n")?))
            })
            .expect("the root to resolve")
        })
      })
      .collect();

    let resolved: Vec<Arc<Ltx>> = workers.into_iter().map(|it| it.join().expect("no panic")).collect();

    assert_eq!(
      produced.load(Ordering::SeqCst),
      1,
      "one resolution for eight racing readers"
    );

    for value in &resolved {
      assert!(Arc::ptr_eq(value, &resolved[0]), "every reader to answer one value");
      assert_eq!(value.get_from("section", "key"), Some("value"));
    }
  }

  #[test]
  fn a_failure_is_not_remembered() -> XrfResult {
    let root: LtxResolvedRoot = LtxResolvedRoot::default();

    assert!(
      root
        .get_or_try_init(|| Err(XrfError::new_convert_error("the source was unreachable")))
        .is_err()
    );

    // The retry is the point: a read that failed once must not leave the root unresolvable for the whole project.
    let resolved: Arc<Ltx> = root.get_or_try_init(|| Ok(Arc::new(Ltx::read_from_str("[later]\nkey = ok\n")?)))?;

    assert_eq!(resolved.get_from("later", "key"), Some("ok"));

    Ok(())
  }
}
