use std::cmp::Reverse;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::Value;
use tauri::ipc::Channel;
use uuid::Uuid;
use xrf_job::{DEFAULT_PROGRESS_INTERVAL, JobHandle, JobProgress, ProgressSink};
use xrf_utils::{format_duration, wall_clock_millis};

use crate::core::jobs::job_conclusion::JobConclusion;
use crate::core::jobs::job_description::JobDescription;
use crate::core::jobs::job_ending::JobEnding;
use crate::core::jobs::job_leases::JobLeases;
use crate::core::jobs::job_progress_sink::JobProgressSink;
use crate::core::jobs::job_registration::JobRegistration;
use crate::core::jobs::job_start::JobStart;
use crate::core::jobs::{JobKind, JobResource};
use crate::core::types::TauriResult;

/// Finished jobs the listing keeps.
const RETAINED_JOBS: usize = 20;

/// Cancellations held for jobs that have not registered yet.
const RETAINED_TOMBSTONES: usize = 32;

/// A job while it is running.
struct LiveJob {
  kind: JobKind,
  lease_keys: Vec<String>,
  started_at: Instant,
  /// The same moment on the wall clock, for a listing that has to say when rather than how long.
  started_at_epoch: u64,
  /// Kept so the registry can both stop the job and read its progress.
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
  fn describe(&self, id: Uuid, ending: Option<JobEnding>) -> JobDescription {
    let (conclusion, error, result) = match ending {
      Some(ending) => (Some(ending.conclusion), ending.error, ending.result),
      None => (None, None, None),
    };

    JobDescription {
      id,
      kind: self.kind,
      lease_keys: self.lease_keys.clone(),
      request: self.request.clone(),
      is_cancel_requested: self.is_cancel_requested,
      progress: self.handle.get_progress(),
      conclusion,
      error,
      result,
      duration: self.started_at.elapsed(),
      started_at: self.started_at_epoch,
    }
  }
}

#[derive(Default)]
struct RegistryState {
  live: HashMap<Uuid, LiveJob>,
  leases: JobLeases,
  finished: VecDeque<JobDescription>,
  /// Cancels that arrived before the job they name, newest last.
  tombstones: VecDeque<Uuid>,
}

impl RegistryState {
  /// Whether a job with this identity is already among the retained finished ones.
  fn has_finished(&self, id: Uuid) -> bool {
    self.finished.iter().any(|finished| finished.id == id)
  }

  /// Retain a finished job, dropping the oldest once the listing is full.
  fn retain_finished(&mut self, description: JobDescription) {
    self.finished.push_back(description);

    while self.finished.len() > RETAINED_JOBS {
      self.finished.pop_front();
    }
  }

  /// Hold a cancel for a job that has not registered yet, dropping the oldest once full.
  fn remember_cancel(&mut self, id: Uuid) {
    if self.tombstones.contains(&id) {
      return;
    }

    self.tombstones.push_back(id);

    while self.tombstones.len() > RETAINED_TOMBSTONES {
      self.tombstones.pop_front();
    }
  }

  /// Whether a cancel was waiting for this job, taking it where it was.
  fn take_tombstone(&mut self, id: Uuid) -> bool {
    match self.tombstones.iter().position(|waiting| *waiting == id) {
      Some(index) => {
        self.tombstones.remove(index);

        true
      }
      None => false,
    }
  }
}

/// Every job the backend is running, what each one holds exclusively, and what recently finished.
pub struct JobRegistry {
  state: Mutex<RegistryState>,
  /// How often a running job is asked where it has got to, and the interval its handles report at.
  interval: Duration,
  /// Whether a thread is already asking, so a second request for reporting is not a second thread.
  is_reporting: AtomicBool,
}

impl JobRegistry {
  /// A registry whose jobs report at the interval `xrf-job` chose.
  pub fn new() -> Self {
    Self::with_interval(DEFAULT_PROGRESS_INTERVAL)
  }

  /// A registry whose jobs report at `interval`.
  pub fn with_interval(interval: Duration) -> Self {
    Self {
      state: Mutex::new(RegistryState::default()),
      interval,
      is_reporting: AtomicBool::new(false),
    }
  }

  /// Take the job's lease keys and start reporting it as running.
  pub fn register(self: &Arc<Self>, start: JobStart) -> TauriResult<(JobHandle, JobRegistration)> {
    let id: Uuid = start.id;
    let kind: JobKind = start.kind;

    self
      .admit(start)
      .inspect(|(handle, _)| match handle.is_cancelled() {
        true => log::info!("Started job {kind} '{id}', already cancelled"),
        false => log::info!("Started job {kind} '{id}'"),
      })
      .inspect_err(|reason| log::warn!("{reason}"))
  }

  /// Take the leases and start reporting the job as running, or answer why it cannot start.
  fn admit(self: &Arc<Self>, start: JobStart) -> TauriResult<(JobHandle, JobRegistration)> {
    let JobStart {
      id,
      kind,
      resources,
      exclusion_group,
      request,
      progress,
    } = start;

    let resources: Vec<JobResource> = resources.iter().map(JobResource::resolve).collect::<TauriResult<_>>()?;
    let mut lease_keys: Vec<String> = resources.iter().map(JobResource::describe).collect();

    if let Some(group) = &exclusion_group {
      lease_keys.push(group.clone());
    }

    let sink: Arc<JobProgressSink> = Arc::new(match progress {
      Some(channel) => JobProgressSink::new(channel),
      None => JobProgressSink::detached(),
    });

    let reporting: Arc<dyn ProgressSink> = Arc::clone(&sink) as Arc<dyn ProgressSink>;
    let handle: JobHandle = JobHandle::with_interval(reporting, self.interval);
    let mut state: MutexGuard<RegistryState> = self.lock();

    if state.live.contains_key(&id) {
      return Err(format!("Job '{id}' has already been registered"));
    }

    if let Some((owner, taken)) = state.leases.find_conflict(exclusion_group.as_deref(), &resources) {
      let holder: Option<&LiveJob> = state.live.get(&owner);

      return Err(format!(
        "Cannot start {kind}: {} is already working on '{taken}'.",
        holder.map_or("another job", |job| job.kind.as_str())
      ));
    }

    // A cancel that arrived first still counts. Without this the frontend would have to know not to offer the control
    // until the backend had acknowledged a job it cannot see.
    if state.take_tombstone(id) {
      handle.cancel();
    }

    state.leases.take(id, exclusion_group, resources);

    let is_cancel_requested: bool = handle.is_cancelled();

    state.live.insert(
      id,
      LiveJob {
        kind,
        lease_keys,
        started_at: Instant::now(),
        started_at_epoch: wall_clock_millis(),
        handle: handle.clone(),
        sink,
        request,
        is_cancel_requested,
      },
    );

    Ok((handle, JobRegistration::new(Arc::clone(self), id)))
  }

  /// Report a running job's progress to `channel` from now on.
  pub fn attach(&self, id: Uuid, channel: Channel<JobProgress>) -> bool {
    let state: MutexGuard<RegistryState> = self.lock();

    let Some(job) = state.live.get(&id) else {
      return false;
    };

    job.sink.attach(channel);

    true
  }

  /// Ask a job to stop, whether or not it has registered yet.
  pub fn cancel(&self, id: Uuid) -> bool {
    let mut state: MutexGuard<RegistryState> = self.lock();

    if let Some(job) = state.live.get_mut(&id) {
      job.is_cancel_requested = true;
      job.handle.cancel();

      let kind: JobKind = job.kind;

      drop(state);

      log::info!("Cancelling job {kind} '{id}'");

      return true;
    }

    // Already finished, or not started yet, and the registry cannot tell which. Recording it is right either way: a
    // tombstone for a job that never comes is discarded, and one for a job still on its way is the whole point.
    if state.has_finished(id) {
      return false;
    }

    state.remember_cancel(id);

    false
  }

  /// Starts asking running jobs where they have got to, and answers with the registry now doing it.
  pub fn start_reporting(self: Arc<Self>) -> Arc<Self> {
    if self.is_reporting.swap(true, Ordering::Relaxed) {
      return self;
    }

    let registry: Weak<Self> = Arc::downgrade(&self);
    let interval: Duration = self.interval;

    thread::spawn(move || {
      loop {
        match registry.upgrade() {
          Some(registry) => registry.report_live(),
          None => break,
        }

        thread::sleep(interval);
      }
    });

    self
  }

  /// Asks every running job to say where it has got to.
  pub(super) fn report_live(&self) {
    let live: Vec<JobHandle> = {
      let state: MutexGuard<RegistryState> = self.lock();

      state.live.values().map(|job| job.handle.clone()).collect()
    };

    for handle in live {
      handle.report_if_due();
    }
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
  pub(super) fn finish(&self, id: Uuid, ending: JobEnding) {
    let mut state: MutexGuard<RegistryState> = self.lock();

    let Some(job) = state.live.remove(&id) else {
      return;
    };

    let kind: JobKind = job.kind;
    let duration: Duration = job.started_at.elapsed();
    let conclusion: JobConclusion = ending.conclusion;
    let error: Option<String> = ending.error.clone();

    state.leases.release(id);
    state.retain_finished(job.describe(id, Some(ending)));

    drop(state);

    let elapsed: String = format_duration(duration);

    // Every way a job can end passes through here, including a panic unwinding out of the command, so this is the one
    // place a run's ending is reported. What it was asked to do was logged by the command that asked.
    match conclusion {
      JobConclusion::Completed => log::info!("Completed job {kind} '{id}' in {elapsed}"),
      JobConclusion::Cancelled => log::info!("Cancelled job {kind} '{id}' after {elapsed}"),
      JobConclusion::Failed => match error {
        Some(error) => log::error!("Failed job {kind} '{id}' after {elapsed}: {error}"),
        None => log::error!("Failed job {kind} '{id}' after {elapsed}, reporting no reason"),
      },
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
