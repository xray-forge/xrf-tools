use std::cmp::Reverse;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use serde::Serialize;
use serde_json::Value;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{JobHandle, JobProgress, ProgressSink};

use crate::core::jobs::job_conclusion::JobConclusion;
use crate::core::jobs::job_description::JobDescription;
use crate::core::jobs::job_leases::JobLeases;
use crate::core::jobs::job_progress_sink::JobProgressSink;
use crate::core::jobs::job_start::JobStart;
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
  /// Where this job's snapshots go, kept so a page that reloaded onto a dead channel can point it at a live one.
  sink: Arc<JobProgressSink>,
  /// What the job was asked to do, as the command described it. Opaque here, and the only record of it once the page
  /// that sent it is gone.
  request: Option<Value>,
  is_cancel_requested: bool,
}

impl LiveJob {
  /// This job as the listing describes it: running where `ending` is absent, and finished where it is.
  ///
  /// One shape for both, because the listing shows one list and a job crossing from running to finished should change
  /// its fields rather than the code that builds them.
  fn describe(&self, id: Uuid, ending: Option<JobEnding>) -> JobDescription {
    let (conclusion, error, result) = match ending {
      Some(ending) => (Some(ending.conclusion), ending.error, ending.result),
      None => (None, None, None),
    };

    JobDescription {
      id,
      kind: self.kind.clone(),
      lease_keys: self.lease_keys.clone(),
      request: self.request.clone(),
      is_cancel_requested: self.is_cancel_requested,
      progress: self.handle.get_progress(),
      conclusion,
      error,
      result,
      duration: self.started_at.elapsed(),
    }
  }
}

/// How a job ended, as the run itself reported it.
struct JobEnding {
  conclusion: JobConclusion,
  error: Option<String>,
  /// What the run answered, serialized by the command that knows its type and never read here.
  ///
  /// Retained because a command's answer is delivered to the caller that asked for it, and after a reload that caller
  /// is gone: without a copy the outcome of a run somebody waited ten minutes for is unrecoverable.
  result: Option<Value>,
}

struct RegistryState {
  live: HashMap<Uuid, LiveJob>,
  leases: JobLeases,
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
        leases: JobLeases::default(),
        finished: VecDeque::new(),
        tombstones: VecDeque::new(),
      }),
    }
  }

  /// Take the job's lease keys and start reporting it as running.
  ///
  /// A key already held refuses the whole registration, naming the holder — never joins it, because two requests for
  /// one destination can carry different configurations, and handing the second caller the first caller's result would
  /// report a success for work nobody asked for. What holding a key means is `JobLeases`.
  ///
  /// Call this before moving the work onto a blocking thread. Registering inside the closure leaves a window in which
  /// a second request sees no holder and both proceed, which is the race the lease exists to close.
  ///
  /// The handle is made here rather than by the caller so that every job's transport is one the registry can re-point
  /// later. A command that built its own sink would produce a job nobody could re-attach to after a reload, and the
  /// gap would only show up as a frozen bar in front of a user.
  ///
  /// What the job carries beyond its identity - what it holds, what it was asked to do, and who is watching - is
  /// described by `start` rather than passed alongside it, because a command knows all of it at once.
  pub fn register(self: &Arc<Self>, start: JobStart) -> TauriResult<(JobHandle, JobRegistration)> {
    let JobStart {
      id,
      kind,
      lease_keys,
      request,
      progress,
    } = start;

    let sink: Arc<JobProgressSink> = Arc::new(match progress {
      Some(channel) => JobProgressSink::new(channel),
      None => JobProgressSink::detached(),
    });
    let reporting: Arc<dyn ProgressSink> = Arc::clone(&sink) as Arc<dyn ProgressSink>;
    let handle: JobHandle = JobHandle::new(reporting);
    let mut state: MutexGuard<RegistryState> = self.lock();

    if let Some(taken) = state.leases.get_taken(&lease_keys) {
      let holder: Option<&LiveJob> = state.leases.get_holder(taken).and_then(|owner| state.live.get(&owner));

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

    state.leases.take(id, &lease_keys);

    let is_cancel_requested: bool = handle.is_cancelled();

    state.live.insert(
      id,
      LiveJob {
        kind,
        lease_keys,
        started_at: Instant::now(),
        handle: handle.clone(),
        sink,
        request,
        is_cancel_requested,
      },
    );

    Ok((
      handle,
      JobRegistration {
        id,
        ending: Mutex::new(None),
        registry: Arc::clone(self),
      },
    ))
  }

  /// Report a running job's progress to `channel` from now on.
  ///
  /// What a reloaded page calls once it has found the jobs it is taking over. Until it does, the job is still writing
  /// snapshots to the channel of a page that no longer exists, which is harmless and noisy in equal measure.
  ///
  /// Answers whether anything is now reporting there. `false` means the job is not running: either it finished while
  /// the page was loading, or it never started, and either way the listing is what describes it.
  pub fn attach(&self, id: Uuid, channel: Channel<JobProgress>) -> bool {
    let state: MutexGuard<RegistryState> = self.lock();

    let Some(job) = state.live.get(&id) else {
      return false;
    };

    job.sink.attach(channel);

    true
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
    let mut running: Vec<JobDescription> = state.live.iter().map(|(id, job)| job.describe(*id, None)).collect();

    // A map has no order of its own, and a listing that reshuffled itself on every read would be unreadable.
    // Longest-running first, so the job somebody is most likely looking for does not move as shorter ones come and go.
    running.sort_by_key(|job| Reverse(job.duration));
    running.extend(state.finished.iter().rev().cloned());

    running
  }

  /// Release a job's leases and move it into the retained listing.
  fn finish(&self, id: Uuid, ending: JobEnding) {
    let mut state: MutexGuard<RegistryState> = self.lock();

    let Some(job) = state.live.remove(&id) else {
      return;
    };

    state.leases.release(id, &job.lease_keys);
    state.finished.push_back(job.describe(id, Some(ending)));

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
  ending: Mutex<Option<JobEnding>>,
}

impl JobRegistration {
  pub fn id(&self) -> Uuid {
    self.id
  }

  /// Record how the job ended, before its leases are released.
  ///
  /// For a run with nothing to hand back. Where there is an answer, `conclude_with` keeps it.
  pub fn conclude(&self, conclusion: JobConclusion, error: Option<String>) {
    self.record(JobEnding {
      conclusion,
      error,
      result: None,
    });
  }

  /// Record the ending a `Result` describes, which is what a command has in hand.
  ///
  /// The success payload is kept as JSON, so a page that was not there when the run finished can still be told what it
  /// answered. Serialization is the same one the command's own response goes through, and a payload that cannot be
  /// serialized is dropped rather than failing the job: the run itself succeeded, and its caller is being told so
  /// through the response either way.
  pub fn conclude_with<T: Serialize, E: ToString>(&self, outcome: &Result<T, E>, cancelled: bool) {
    let ending: JobEnding = match outcome {
      Ok(value) => JobEnding {
        conclusion: if cancelled {
          JobConclusion::Cancelled
        } else {
          JobConclusion::Completed
        },
        error: None,
        result: serde_json::to_value(value)
          .inspect_err(|error| log::warn!("Job answer was not retained: {error}"))
          .ok(),
      },
      Err(error) => JobEnding {
        conclusion: JobConclusion::Failed,
        error: Some(error.to_string()),
        result: None,
      },
    };

    self.record(ending);
  }

  fn record(&self, ending: JobEnding) {
    *self.ending.lock().unwrap_or_else(PoisonError::into_inner) = Some(ending);
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
    let ending: JobEnding = self
      .ending
      .lock()
      .unwrap_or_else(PoisonError::into_inner)
      .take()
      .unwrap_or(JobEnding {
        conclusion: JobConclusion::Failed,
        error: None,
        result: None,
      });

    self.registry.finish(self.id, ending);
  }
}
