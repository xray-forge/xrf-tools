use std::sync::{Arc, Barrier};
use std::thread;

use uuid::Uuid;
use xrf_job::JobHandle;

use crate::core::jobs::job_conclusion::JobConclusion;
use crate::core::jobs::{JobRegistration, JobRegistry, JobStart};

#[test]
fn grouped_modes_stay_exclusive_until_cancelled_work_settles() {
  for first_kind in ["format", "check-format"] {
    let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
    let id: Uuid = Uuid::new_v4();
    let (_job, registration): (JobHandle, JobRegistration) = registry
      .register(
        JobStart::new(id, first_kind)
          .with_exclusion_group("formatter")
          .with_lease_keys(vec!["output:a".to_owned()]),
      )
      .expect("first mode starts");

    for is_cancelled in [false, true] {
      if is_cancelled {
        registry.cancel(id);
      }

      for second_kind in ["format", "check-format"] {
        assert!(
          registry
            .register(
              JobStart::new(Uuid::new_v4(), second_kind)
                .with_exclusion_group("formatter")
                .with_lease_keys(vec!["output:b".to_owned()]),
            )
            .is_err(),
          "{second_kind} must not overlap {first_kind}, even at a different destination"
        );
      }
    }

    registration.conclude(JobConclusion::Cancelled, None);
    drop(registration);

    registry
      .register(JobStart::new(Uuid::new_v4(), "format").with_exclusion_group("formatter"))
      .expect("settlement releases the group");
  }
}

#[test]
fn resource_leases_remain_exclusive_across_different_groups() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let (_job, _registration): (JobHandle, JobRegistration) = registry
    .register(
      JobStart::new(Uuid::new_v4(), "build")
        .with_exclusion_group("builder")
        .with_lease_keys(vec!["output:shared".to_owned()]),
    )
    .expect("builder starts");

  assert!(
    registry
      .register(
        JobStart::new(Uuid::new_v4(), "parse")
          .with_lease_keys(vec!["output:shared".to_owned()])
          .with_exclusion_group("parser"),
      )
      .is_err()
  );

  registry
    .register(
      JobStart::new(Uuid::new_v4(), "parse")
        .with_exclusion_group("parser")
        .with_lease_keys(vec!["output:other".to_owned()]),
    )
    .expect("a refused registration does not retain the free group lease");
}

#[test]
fn failed_work_releases_its_group() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let (_job, registration): (JobHandle, JobRegistration) = registry
    .register(JobStart::new(Uuid::new_v4(), "verify").with_exclusion_group("verifier"))
    .expect("verification starts");

  drop(registration);

  assert_eq!(registry.list()[0].conclusion, Some(JobConclusion::Failed));
  registry
    .register(JobStart::new(Uuid::new_v4(), "verify").with_exclusion_group("verifier"))
    .expect("failure permits a retry");
}

#[test]
fn concurrent_registrations_admit_exactly_one_group_owner() {
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let barrier: Arc<Barrier> = Arc::new(Barrier::new(2));
  let runs: Vec<_> = (0..2)
    .map(|_| {
      let registry: Arc<JobRegistry> = Arc::clone(&registry);
      let barrier: Arc<Barrier> = Arc::clone(&barrier);

      thread::spawn(move || {
        barrier.wait();

        registry.register(JobStart::new(Uuid::new_v4(), "verify").with_exclusion_group("verifier"))
      })
    })
    .collect();
  let outcomes: Vec<_> = runs
    .into_iter()
    .map(|run| run.join().expect("registration thread"))
    .collect();

  assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
}
