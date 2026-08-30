use std::cmp::Reverse;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use uuid::Uuid;
use xrf_job::JobHandle;

use crate::core::jobs::job_conclusion::JobConclusion;
use crate::core::jobs::job_description::JobDescription;
use crate::core::types::TauriResult;

/// Finished jobs the listing keeps.
///
/// A job that vanished the instant it ended would be invisible in exactly the moment somebody wants to look at it:
/// what happened is a question asked after the fact, not during. Bounded because this is a debugging surface and not
/// a history.
const RETAINED_JOBS: usize = 20;

/// Cancellations held for jobs that have not registered yet.
///
/// Small on purpose. One of these is only ever outstanding for the few milliseconds between the frontend sending a
/// cancel and the command it names reaching registration, so a backlog means cancels are being sent for jobs that
/// never start, and keeping them all would be keeping rubbish.
const RETAINED_TOMBSTONES: usize = 32;

/// A job while it is running.
struct LiveJob {
  kind: String,
  lease_keys: Vec<String>,
  started_at: Instant,
  /// Kept so the registry can both stop the job and read its progress.
  ///
  /// Reading it here rather than storing snapshots as they are emitted is what keeps the reporting path free of the
  /// registry: progress goes to whoever is watching, and the listing asks the handle when somebody actually looks.
  handle: JobHandle,
  is_cancel_requested: bool,
}

struct RegistryState {
  live: HashMap<Uuid, LiveJob>,
  /// Lease key to the job holding it, so a refusal can name what refused it.
  leases: HashMap<String, Uuid>,
  finished: VecDeque<JobDescription>,
  /// Cancels that arrived before the job they name, newest last.
  tombstones: VecDeque<Uuid>,
}

/// Every job the backend is running, what each one holds exclusively, and what recently finished.
///
/// One registry for every domain rather than one per plugin: identity, cancellation and exclusion are the same three
/// questions whatever the work is, and a second copy of them is a second set of rules to keep in step.
pub struct JobRegistry {
  state: Mutex<RegistryState>,
}

impl JobRegistry {
  pub fn new() -> Self {
    Self {
      state: Mutex::new(RegistryState {
        live: HashMap::new(),
        leases: HashMap::new(),
        finished: VecDeque::new(),
        tombstones: VecDeque::new(),
      }),
    }
  }

  /// Take `lease_keys` for `id` and start reporting it as running.
  ///
  /// All the keys or none of them: a job that took some and failed on the rest would leave the ones it took held by
  /// nobody. A key already held refuses the whole registration, naming the holder — never joins it, because two
  /// requests for one destination can carry different configurations, and handing the second caller the first
  /// caller's result would report a success for work nobody asked for.
  ///
  /// Call this before moving the work onto a blocking thread. Registering inside the closure leaves a window in which
  /// a second request sees no holder and both proceed, which is the race the lease exists to close.
  pub fn register(
    self: &Arc<Self>,
    id: Uuid,
    kind: impl Into<String>,
    lease_keys: Vec<String>,
    handle: JobHandle,
  ) -> TauriResult<JobRegistration> {
    let kind: String = kind.into();
    let mut state: MutexGuard<RegistryState> = self.lock();

    if let Some(taken) = lease_keys.iter().find(|key| state.leases.contains_key(*key)) {
      let holder: Option<&LiveJob> = state.leases.get(taken).and_then(|owner| state.live.get(owner));

      return Err(format!(
        "Cannot start {kind}: {} is already working on '{taken}'.",
        holder.map_or("another job", |job| job.kind.as_str())
      ));
    }

    // A cancel that arrived first still counts. Without this the frontend would have to know not to offer the control
    // until the backend had acknowledged a job it cannot see.
    if let Some(index) = state.tombstones.iter().position(|waiting| *waiting == id) {
      state.tombstones.remove(index);
      handle.cancel();
    }

    for key in &lease_keys {
      state.leases.insert(key.clone(), id);
    }

    let is_cancel_requested: bool = handle.is_cancelled();

    state.live.insert(
      id,
      LiveJob {
        kind,
        lease_keys,
        started_at: Instant::now(),
        handle,
        is_cancel_requested,
      },
    );

    Ok(JobRegistration {
      id,
      conclusion: Mutex::new(None),
      registry: Arc::clone(self),
    })
  }

  /// Ask a job to stop, whether or not it has registered yet.
  ///
  /// Answers whether anything is now expected to stop, so a caller can tell a cancel that landed from one aimed at a
  /// job that has already finished.
  pub fn cancel(&self, id: Uuid) -> bool {
    let mut state: MutexGuard<RegistryState> = self.lock();

    if let Some(job) = state.live.get_mut(&id) {
      job.is_cancel_requested = true;
      job.handle.cancel();

      return true;
    }

    // Already finished, or not started yet, and the registry cannot tell which. Recording it is right either way: a
    // tombstone for a job that never comes is discarded, and one for a job still on its way is the whole point.
    if state.finished.iter().any(|finished| finished.id == id) {
      return false;
    }

    if !state.tombstones.contains(&id) {
      state.tombstones.push_back(id);

      while state.tombstones.len() > RETAINED_TOMBSTONES {
        state.tombstones.pop_front();
      }
    }

    false
  }

  /// Every job, running first and newest finished after them.
  pub fn list(&self) -> Vec<JobDescription> {
    let state: MutexGuard<RegistryState> = self.lock();
    let mut running: Vec<JobDescription> = state
      .live
      .iter()
      .map(|(id, job)| JobDescription {
        id: *id,
        kind: job.kind.clone(),
        lease_keys: job.lease_keys.clone(),
        is_cancel_requested: job.is_cancel_requested,
        progress: job.handle.get_progress(),
        conclusion: None,
        error: None,
        duration: job.started_at.elapsed(),
      })
      .collect();

    // A map has no order of its own, and a listing that reshuffled itself on every read would be unreadable.
    // Longest-running first, so the job somebody is most likely looking for does not move as shorter ones come and go.
    running.sort_by_key(|job| Reverse(job.duration));
    running.extend(state.finished.iter().rev().cloned());

    running
  }

  /// Release a job's leases and move it into the retained listing.
  fn finish(&self, id: Uuid, conclusion: JobConclusion, error: Option<String>) {
    let mut state: MutexGuard<RegistryState> = self.lock();

    let Some(job) = state.live.remove(&id) else {
      return;
    };

    for key in &job.lease_keys {
      // Only where this job still holds it. Removing by key alone would let a job's teardown release a lease a later
      // job had already taken, which is worse than never having held one.
      if state.leases.get(key) == Some(&id) {
        state.leases.remove(key);
      }
    }

    state.finished.push_back(JobDescription {
      id,
      kind: job.kind,
      lease_keys: job.lease_keys,
      is_cancel_requested: job.is_cancel_requested,
      progress: job.handle.get_progress(),
      conclusion: Some(conclusion),
      error,
      duration: job.started_at.elapsed(),
    });

    while state.finished.len() > RETAINED_JOBS {
      state.finished.pop_front();
    }
  }

  /// A poisoned registry is still the only record of what is running and what it holds. Refusing to answer would turn
  /// one panicking command into an application that can never start another job.
  fn lock(&self) -> MutexGuard<'_, RegistryState> {
    self.state.lock().unwrap_or_else(PoisonError::into_inner)
  }
}

impl Default for JobRegistry {
  fn default() -> Self {
    Self::new()
  }
}

/// A registered job, and the leases it holds, for as long as this is alive.
///
/// The leases are released by dropping this rather than by a call at the end of the command, so a path that returns
/// early — a refused input, an error, a panic — cannot leave a destination owned by a job that is no longer running.
/// Holding it past the work is deliberate too: a cancelled pack keeps its destination until whatever it wrote has been
/// discarded or promoted.
pub struct JobRegistration {
  registry: Arc<JobRegistry>,
  id: Uuid,
  conclusion: Mutex<Option<(JobConclusion, Option<String>)>>,
}

impl JobRegistration {
  pub fn id(&self) -> Uuid {
    self.id
  }

  /// Record how the job ended, before its leases are released.
  pub fn conclude(&self, conclusion: JobConclusion, error: Option<String>) {
    *self.conclusion.lock().unwrap_or_else(PoisonError::into_inner) = Some((conclusion, error));
  }

  /// Record the ending a `Result` describes, which is what a command has in hand.
  pub fn conclude_with<T, E: ToString>(&self, outcome: &Result<T, E>, cancelled: bool) {
    match outcome {
      Ok(_) if cancelled => self.conclude(JobConclusion::Cancelled, None),
      Ok(_) => self.conclude(JobConclusion::Completed, None),
      Err(error) => self.conclude(JobConclusion::Failed, Some(error.to_string())),
    }
  }
}

impl std::fmt::Debug for JobRegistration {
  /// The identity and nothing else: the registry behind this is shared mutable state, and rendering it would mean
  /// taking a lock inside a formatter, which is how a diagnostic turns into a deadlock.
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.debug_struct("JobRegistration").field("id", &self.id).finish()
  }
}

impl Drop for JobRegistration {
  /// Releases the leases and retains the job, concluding it as failed where nobody said otherwise.
  ///
  /// Failed rather than completed, because the paths that reach here without concluding are the ones that went wrong:
  /// an early return, or a panic unwinding through the command. A job that ended well says so.
  fn drop(&mut self) {
    let (conclusion, error) = self
      .conclusion
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .take()
      .unwrap_or((JobConclusion::Failed, None));

    self.registry.finish(self.id, conclusion, error);
  }
}
