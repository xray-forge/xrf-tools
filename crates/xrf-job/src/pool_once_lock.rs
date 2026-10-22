use std::sync::OnceLock;

use rayon::ThreadPoolBuilder;

/// A value built once and shared, which the workers of a Rayon pool may wait on while its build fans out over Rayon.
///
/// A plain [`OnceLock`] deadlocks there. A worker waiting inside the build's `join` takes the pool's other jobs
/// meanwhile, and one that takes a job waiting on the lock its own thread is filling never returns; a worker of any
/// pool such a job runs on does the same. This lock builds on a pool of one thread of its own, which no job waiting on
/// the lock runs on, while the caller's worker sleeps without taking jobs: the build stands in for that worker, so it
/// runs within the caller's plan, though not in parallel.
///
/// Each build starts a thread, so a value whose build never reaches Rayon keeps a plain [`OnceLock`].
pub struct PoolOnceLock<T> {
  cell: OnceLock<T>,
}

impl<T> PoolOnceLock<T> {
  pub const fn new() -> Self {
    Self { cell: OnceLock::new() }
  }

  /// The value, once built.
  pub fn get(&self) -> Option<&T> {
    self.cell.get()
  }
}

impl<T: Send> PoolOnceLock<T> {
  /// The value, built by `build` the first time anything asks; an ask made while it is built waits for it, from a
  /// worker of any pool or from none.
  ///
  /// # Panics
  ///
  /// Panics where `build` panics, as [`OnceLock::get_or_init`] does, leaving the value unbuilt.
  pub fn get_or_init(&self, build: impl FnOnce() -> T + Send) -> &T {
    self.cell.get_or_init(|| Self::build_apart(build))
  }

  fn build_apart(build: impl FnOnce() -> T + Send) -> T {
    // `install` from a worker of another pool waits by taking that pool's jobs, so it is called from a plain thread.
    std::thread::scope(|scope| {
      scope
        .spawn(|| {
          match ThreadPoolBuilder::new()
            .num_threads(1)
            .thread_name(|_| "xrf-once".to_string())
            .build()
          {
            Ok(pool) => pool.install(build),
            // Off every pool, the build's parallel work goes to Rayon's global pool. Where that pool's own workers are
            // what wait here, it can deadlock as an unmarked build would; only a thread that cannot start leaves it so.
            Err(error) => {
              log::warn!("Building a shared value on Rayon's global pool, as no thread of its own starts: {error}");

              build()
            }
          }
        })
        .join()
        .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
  }
}

impl<T> Default for PoolOnceLock<T> {
  fn default() -> Self {
    Self::new()
  }
}
