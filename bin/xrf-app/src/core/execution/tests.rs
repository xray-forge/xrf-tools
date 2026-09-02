use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Mutex;
use std::thread;

use xrf_job::{ExecutionOrigin, ExecutionRequest};

use crate::core::execution::execution_state::ExecutionState;

fn bounded_to(workers: usize) -> ExecutionState {
  ExecutionState::new(ExecutionRequest::Workers(
    NonZeroUsize::new(workers).expect("test worker counts are not zero"),
  ))
  .expect("the pool starts")
}

#[test]
fn resolves_the_plan_it_was_asked_for() {
  let execution: ExecutionState = bounded_to(3);

  assert_eq!(execution.get_plan().get_workers().get(), 3);
  assert_eq!(execution.get_plan().get_origin(), ExecutionOrigin::Requested);
}

/// The guarantee the shared pool exists for: concurrent jobs share one budget rather than each taking a whole one.
///
/// Asserted two ways, because the interesting failure passes the first on its own. Every caller seeing the plan's width
/// would also hold if each had built a pool of that size; what rules that out is that the threads doing the work are
/// the *same* threads, so the union across all of them never exceeds the plan.
#[test]
fn bounds_every_concurrent_job_to_one_shared_budget() {
  const WORKERS: usize = 3;
  const JOBS: usize = 8;

  let execution: ExecutionState = bounded_to(WORKERS);
  let widths: Mutex<Vec<usize>> = Mutex::new(Vec::new());
  let workers: Mutex<HashSet<thread::ThreadId>> = Mutex::new(HashSet::new());

  thread::scope(|jobs| {
    for _ in 0..JOBS {
      jobs.spawn(|| {
        execution.get_pool().install(|| {
          widths
            .lock()
            .expect("widths are not poisoned")
            .push(rayon::current_num_threads());

          // Named on every worker the pool actually used for this job, so the set below is the real thread population
          // rather than whichever one happened to answer first.
          rayon::broadcast(|_| {
            workers
              .lock()
              .expect("workers are not poisoned")
              .insert(thread::current().id());
          });
        });
      });
    }
  });

  let widths: Vec<usize> = widths.into_inner().expect("widths are not poisoned");

  assert_eq!(widths.len(), JOBS, "every job ran");
  assert!(
    widths.iter().all(|width| *width == WORKERS),
    "every job saw the plan's width, got {widths:?}"
  );

  let workers: HashSet<thread::ThreadId> = workers.into_inner().expect("workers are not poisoned");

  assert_eq!(
    workers.len(),
    WORKERS,
    "{JOBS} concurrent jobs ran on {} threads, which is not one shared pool of {WORKERS}",
    workers.len()
  );
}
