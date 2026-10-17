use std::sync::mpsc;
use std::time::Duration;

use crate::tests::test_workers::{TEST_WORKER_NAME, create_workers};
use crate::thread::frame_task::FrameTask;
use crate::thread::render_workers::RenderWorkers;

/// Who ran a task: whether it was told it may spread over the pool, and the name of the thread it ran on.
fn report(is_parallel: bool) -> (bool, Option<String>) {
  (is_parallel, std::thread::current().name().map(str::to_owned))
}

#[test]
fn a_task_no_worker_took_runs_on_the_thread_joining_it() {
  let workers: RenderWorkers = create_workers();
  let mut releases: Vec<mpsc::Sender<()>> = Vec::new();
  let (started_sender, started) = mpsc::channel::<()>();

  // Both of the test pool's workers held, so neither can take the task before it is joined.
  for _ in 0..2 {
    let (release, held) = mpsc::channel::<()>();
    let started_sender: mpsc::Sender<()> = started_sender.clone();

    releases.push(release);
    workers.spawn(move || {
      let _ = started_sender.send(());
      let _ = held.recv_timeout(Duration::from_secs(5));
    });
  }

  for _ in 0..2 {
    started.recv_timeout(Duration::from_secs(5)).expect("a worker holds");
  }

  let task: FrameTask<(bool, Option<String>)> = workers.spawn_frame_task(report);
  let (is_parallel, thread) = task.join().expect("the task answers");

  drop(releases);

  assert!(!is_parallel);
  assert!(!thread.is_some_and(|name| name.starts_with(TEST_WORKER_NAME)));
}

#[test]
fn a_task_a_worker_took_is_waited_for() {
  let workers: RenderWorkers = create_workers();
  let (taken_sender, taken) = mpsc::channel::<()>();
  let task: FrameTask<(bool, Option<String>)> = workers.spawn_frame_task(move |is_parallel| {
    let _ = taken_sender.send(());

    std::thread::sleep(Duration::from_millis(20));

    report(is_parallel)
  });

  taken
    .recv_timeout(Duration::from_secs(5))
    .expect("a worker takes the task");

  let (is_parallel, thread) = task.join().expect("the task answers");

  assert!(is_parallel);
  assert!(thread.is_some_and(|name| name.starts_with(TEST_WORKER_NAME)));
}
