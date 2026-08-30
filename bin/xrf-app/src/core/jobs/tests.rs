use std::sync::Arc;

use uuid::Uuid;
use xrf_job::JobHandle;

use crate::core::jobs::job_conclusion::JobConclusion;
use crate::core::jobs::job_description::JobDescription;
use crate::core::jobs::job_registry::{JobRegistration, JobRegistry};

fn registry() -> Arc<JobRegistry> {
  Arc::new(JobRegistry::new())
}

fn keys(keys: &[&str]) -> Vec<String> {
  keys.iter().map(|key| String::from(*key)).collect()
}

#[test]
fn a_registered_job_is_listed_as_running() {
  let registry: Arc<JobRegistry> = registry();
  let id: Uuid = Uuid::new_v4();

  let _running: JobRegistration = registry
    .register(
      id,
      "archives.pack",
      keys(&["pack:c:\\out|gamedata"]),
      JobHandle::inert(),
    )
    .expect("nothing else holds the destination");

  let listed: Vec<JobDescription> = registry.list();

  assert_eq!(listed.len(), 1);
  assert_eq!(listed[0].id, id);
  assert_eq!(listed[0].kind, "archives.pack");
  assert_eq!(listed[0].conclusion, None);
  assert!(!listed[0].is_cancel_requested);
}

#[test]
fn a_held_lease_key_refuses_the_second_job_and_names_the_first() {
  // The refusal has to say what refused it: a bare "busy" leaves the user with no way to find the run they forgot.
  let registry: Arc<JobRegistry> = registry();

  let _first: JobRegistration = registry
    .register(
      Uuid::new_v4(),
      "archives.pack",
      keys(&["pack:c:\\out|gamedata"]),
      JobHandle::inert(),
    )
    .expect("the first job takes the destination");

  let refused: String = registry
    .register(
      Uuid::new_v4(),
      "archives.pack",
      keys(&["pack:c:\\out|gamedata"]),
      JobHandle::inert(),
    )
    .expect_err("the destination is taken");

  assert!(refused.contains("archives.pack"), "names the holder, got {refused:?}");
  assert!(
    refused.contains("pack:c:\\out|gamedata"),
    "names what is held, got {refused:?}"
  );
}

#[test]
fn an_unrelated_destination_runs_alongside() {
  // Exclusion is per key, not per kind. Two packs writing different sets have no reason to queue behind each other.
  let registry: Arc<JobRegistry> = registry();

  let _first: JobRegistration = registry
    .register(
      Uuid::new_v4(),
      "archives.pack",
      keys(&["pack:c:\\a"]),
      JobHandle::inert(),
    )
    .expect("first destination");

  let _second: JobRegistration = registry
    .register(
      Uuid::new_v4(),
      "archives.pack",
      keys(&["pack:c:\\b"]),
      JobHandle::inert(),
    )
    .expect("a different destination is not held");

  assert_eq!(registry.list().len(), 2);
}

#[test]
fn a_singleton_is_a_constant_key() {
  // No policy enum: one-at-a-time is what you get by keying on the kind itself.
  let registry: Arc<JobRegistry> = registry();

  let _first: JobRegistration = registry
    .register(
      Uuid::new_v4(),
      "gamedata.verify",
      keys(&["gamedata.verify"]),
      JobHandle::inert(),
    )
    .expect("the first verification");

  assert!(
    registry
      .register(
        Uuid::new_v4(),
        "gamedata.verify",
        keys(&["gamedata.verify"]),
        JobHandle::inert()
      )
      .is_err(),
    "a constant key admits one job at a time"
  );
}

#[test]
fn a_refused_registration_takes_none_of_its_keys() {
  // All or nothing. A job that took the free keys and failed on the held one would leave them owned by nobody, which
  // is worse than the refusal it was reporting.
  let registry: Arc<JobRegistry> = registry();

  let _holder: JobRegistration = registry
    .register(Uuid::new_v4(), "archives.pack", keys(&["taken"]), JobHandle::inert())
    .expect("the holder");

  assert!(
    registry
      .register(
        Uuid::new_v4(),
        "archives.pack",
        keys(&["free", "taken"]),
        JobHandle::inert()
      )
      .is_err()
  );

  registry
    .register(Uuid::new_v4(), "archives.pack", keys(&["free"]), JobHandle::inert())
    .expect("the free key was never taken by the refused registration");
}

#[test]
fn dropping_a_registration_releases_its_leases() {
  let registry: Arc<JobRegistry> = registry();

  {
    let _running: JobRegistration = registry
      .register(
        Uuid::new_v4(),
        "archives.pack",
        keys(&["pack:c:\\out"]),
        JobHandle::inert(),
      )
      .expect("the first job");
  }

  registry
    .register(
      Uuid::new_v4(),
      "archives.pack",
      keys(&["pack:c:\\out"]),
      JobHandle::inert(),
    )
    .expect("the destination was released with the job");
}

#[test]
fn a_concluded_job_is_retained_with_its_outcome() {
  let registry: Arc<JobRegistry> = registry();
  let id: Uuid = Uuid::new_v4();

  {
    let running: JobRegistration = registry
      .register(id, "archives.unpack", Vec::new(), JobHandle::inert())
      .expect("nothing is held");

    running.conclude(JobConclusion::Completed, None);
  }

  let listed: Vec<JobDescription> = registry.list();

  assert_eq!(listed.len(), 1);
  assert_eq!(listed[0].id, id);
  assert_eq!(listed[0].conclusion, Some(JobConclusion::Completed));
}

#[test]
fn a_registration_dropped_without_a_conclusion_is_recorded_as_failed() {
  // The paths that reach a drop without concluding are the ones that went wrong: an early return, or a panic
  // unwinding through the command. Recording those as completed would be reporting a success nobody claimed.
  let registry: Arc<JobRegistry> = registry();

  drop(
    registry
      .register(Uuid::new_v4(), "archives.pack", Vec::new(), JobHandle::inert())
      .expect("nothing is held"),
  );

  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Failed));
}

#[test]
fn a_failed_conclusion_carries_why() {
  let registry: Arc<JobRegistry> = registry();

  {
    let running: JobRegistration = registry
      .register(Uuid::new_v4(), "archives.pack", Vec::new(), JobHandle::inert())
      .expect("nothing is held");

    running.conclude_with::<(), String>(&Err(String::from("volume cap refuses particles.xr")), false);
  }

  let listed: Vec<JobDescription> = registry.list();

  assert_eq!(listed[0].conclusion, Some(JobConclusion::Failed));
  assert_eq!(listed[0].error.as_deref(), Some("volume cap refuses particles.xr"));
}

#[test]
fn a_result_that_succeeded_after_a_cancellation_concludes_as_cancelled() {
  // A cancelled operation answers `Ok` carrying what it managed to do, so the outcome cannot be read off the result
  // alone — the registry is told separately whether stopping was asked for.
  let registry: Arc<JobRegistry> = registry();

  {
    let running: JobRegistration = registry
      .register(Uuid::new_v4(), "archives.unpack", Vec::new(), JobHandle::inert())
      .expect("nothing is held");

    running.conclude_with::<u8, String>(&Ok(1), true);
  }

  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Cancelled));
}

#[test]
fn cancelling_a_running_job_reaches_its_handle() {
  let registry: Arc<JobRegistry> = registry();
  let id: Uuid = Uuid::new_v4();
  let handle: JobHandle = JobHandle::inert();

  let _running: JobRegistration = registry
    .register(id, "archives.unpack", Vec::new(), handle.clone())
    .expect("nothing is held");

  assert!(registry.cancel(id), "a running job is expected to stop");
  assert!(handle.is_cancelled());
  assert!(registry.list()[0].is_cancel_requested);
}

#[test]
fn a_cancel_that_arrives_before_its_job_still_lands() {
  // The frontend knows a job's identity before the command carrying it is sent, so a cancel can legitimately arrive
  // first. Without the tombstone the control would have to be withheld until the backend acknowledged the job.
  let registry: Arc<JobRegistry> = registry();
  let id: Uuid = Uuid::new_v4();
  let handle: JobHandle = JobHandle::inert();

  assert!(!registry.cancel(id), "nothing is running under that identity yet");

  let _running: JobRegistration = registry
    .register(id, "archives.pack", Vec::new(), handle.clone())
    .expect("nothing is held");

  assert!(handle.is_cancelled(), "the waiting cancel was applied at registration");
  assert!(registry.list()[0].is_cancel_requested);
}

#[test]
fn a_tombstone_is_spent_by_the_job_it_named() {
  // Otherwise a stale cancel would keep killing later jobs that happened to reuse the identity, which is the stale-run
  // hazard the fresh-id rule exists to avoid.
  let registry: Arc<JobRegistry> = registry();
  let id: Uuid = Uuid::new_v4();

  registry.cancel(id);

  drop(
    registry
      .register(id, "archives.pack", Vec::new(), JobHandle::inert())
      .expect("nothing is held"),
  );

  let second: JobHandle = JobHandle::inert();

  let _running: JobRegistration = registry
    .register(id, "archives.pack", Vec::new(), second.clone())
    .expect("nothing is held");

  assert!(
    !second.is_cancelled(),
    "the cancel belonged to the run that consumed it"
  );
}

#[test]
fn cancelling_a_finished_job_answers_no_and_leaves_nothing_waiting() {
  let registry: Arc<JobRegistry> = registry();
  let id: Uuid = Uuid::new_v4();

  {
    let running: JobRegistration = registry
      .register(id, "archives.pack", Vec::new(), JobHandle::inert())
      .expect("nothing is held");

    running.conclude(JobConclusion::Completed, None);
  }

  assert!(!registry.cancel(id), "the job has already finished");

  // Nothing was left waiting: registering that identity again is not born cancelled.
  let handle: JobHandle = JobHandle::inert();
  let _running: JobRegistration = registry
    .register(id, "archives.pack", Vec::new(), handle.clone())
    .expect("nothing is held");

  assert!(!handle.is_cancelled());
}

#[test]
fn the_retained_listing_is_bounded_and_keeps_the_newest() {
  // A debugging surface, not a history. What matters is that it does not grow without limit and that the last thing
  // that happened is still in it.
  let registry: Arc<JobRegistry> = registry();

  for index in 0..40 {
    let running: JobRegistration = registry
      .register(Uuid::new_v4(), format!("kind.{index}"), Vec::new(), JobHandle::inert())
      .expect("nothing is held");

    running.conclude(JobConclusion::Completed, None);
  }

  let listed: Vec<JobDescription> = registry.list();

  assert_eq!(listed.len(), 20);
  assert_eq!(listed[0].kind, "kind.39", "newest finished job first");
  assert_eq!(listed[19].kind, "kind.20");
}

#[test]
fn a_finished_job_never_releases_a_lease_a_later_job_holds() {
  // Releasing by key alone would let one job's teardown hand away a destination somebody else had already taken, which
  // is worse than never having held a lease at all. Contrived only in its timing: it is what an out-of-order drop does.
  let registry: Arc<JobRegistry> = registry();
  let first: JobRegistration = registry
    .register(
      Uuid::new_v4(),
      "archives.pack",
      keys(&["pack:c:\\out"]),
      JobHandle::inert(),
    )
    .expect("the first job");

  drop(first);

  let _second: JobRegistration = registry
    .register(
      Uuid::new_v4(),
      "archives.pack",
      keys(&["pack:c:\\out"]),
      JobHandle::inert(),
    )
    .expect("released with the first job");

  assert!(
    registry
      .register(
        Uuid::new_v4(),
        "archives.pack",
        keys(&["pack:c:\\out"]),
        JobHandle::inert()
      )
      .is_err(),
    "the second job still holds it"
  );
}

#[test]
fn a_running_job_reports_the_progress_of_its_own_handle() {
  // Read on demand rather than pushed: nothing has to reach the registry for a listing to be current.
  let registry: Arc<JobRegistry> = registry();
  let handle: JobHandle = JobHandle::inert();

  let _running: JobRegistration = registry
    .register(Uuid::new_v4(), "archives.unpack", Vec::new(), handle.clone())
    .expect("nothing is held");

  assert!(registry.list()[0].progress.is_none(), "no level has been entered yet");

  let writing = handle.enter("write", Some(10));

  writing.advance();

  let listed: Vec<JobDescription> = registry.list();
  let progress = listed[0].progress.as_ref().expect("a level is entered");

  assert_eq!(progress.levels[0].id, "write");
  assert_eq!(progress.levels[0].completed, 1);
}
