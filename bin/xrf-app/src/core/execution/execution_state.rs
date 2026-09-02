use std::sync::Arc;

use rayon::ThreadPool;
use xrf_error::XrfResult;
use xrf_job::{ExecutionPlan, ExecutionRequest};

use crate::core::types::TauriResult;

/// The one pool this application runs bounded work on, and the plan describing it.
///
/// One pool for the process rather than one per job. A pool per job would multiply the plan by however many jobs happen
/// to be running, so unpacking two archives at once would ask for twice the machine and three at once for three times
/// it — a width nobody chose and nothing states. Rayon steals work across whatever is queued, so jobs sharing this pool
/// interleave rather than queue behind each other, and the total stays what the plan says however many are alive.
///
/// Held behind an `Arc` for the reason the job registry is: the pool travels onto a blocking thread and has to outlive
/// the command frame that reached for it.
pub struct ExecutionState {
  plan: ExecutionPlan,
  pool: Arc<ThreadPool>,
}

impl ExecutionState {
  /// Resolves `request` against this host and starts the pool it describes.
  ///
  /// # Errors
  ///
  /// Returns an error when the threads cannot be started, which leaves the application with nothing to run bounded work
  /// on and is therefore a startup failure rather than something to recover from per command.
  pub fn new(request: ExecutionRequest) -> XrfResult<Self> {
    let plan: ExecutionPlan = request.resolve();

    Ok(Self {
      plan,
      pool: Arc::new(plan.build_pool()?),
    })
  }

  /// The width every job on this pool shares.
  pub const fn get_plan(&self) -> &ExecutionPlan {
    &self.plan
  }

  /// Runs synchronous work off the executor and inside this application's pool.
  ///
  /// Both halves matter and neither is optional. `async fn` alone moves nothing off the IPC executor, so work bounded
  /// by input size has to cross to a blocking thread; and work that crosses without installing lands on Rayon's global
  /// pool, where the plan bounds nothing. Going through here is what makes the width a property of the application
  /// rather than of whichever command remembered to ask for it.
  ///
  /// `what` names the operation for the one failure this can add — the blocking thread not finishing — and is the only
  /// thing this reports on. Whatever `work` answers with is handed back untouched, so a command keeps its own result
  /// shaping and its own errors.
  ///
  /// # Errors
  ///
  /// Returns an error when the blocking thread panics or is cancelled before answering.
  pub async fn run_blocking<T, F>(&self, what: &str, work: F) -> TauriResult<T>
  where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
  {
    let pool: Arc<ThreadPool> = self.get_pool();

    tauri::async_runtime::spawn_blocking(move || pool.install(work))
      .await
      .map_err(|error| format!("{what} did not finish: {error}"))
  }

  /// A handle on the pool, for the callers that install into it themselves.
  ///
  /// A handle rather than a borrow because the pool outlives the frame that reached for it: the hop below moves it onto
  /// a blocking thread, and an assertion about the bound installs into it without a hop at all.
  pub(super) fn get_pool(&self) -> Arc<ThreadPool> {
    Arc::clone(&self.pool)
  }
}
