use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::pool_once_lock::PoolOnceLock;

/// Long enough for any step here on a loaded machine; reached only where the lock deadlocked.
const WAIT: Duration = Duration::from_secs(10);
/// How long the build's first half gives the second to be taken by another worker, which a pool of one never has.
const TAKEN_WAIT: Duration = Duration::from_millis(200);

fn create_pool(workers: usize) -> ThreadPool {
  ThreadPoolBuilder::new()
    .num_threads(workers)
    .build()
    .expect("a test pool starts")
}

// The order a plain `OnceLock` never returns from: the build's second half is taken by the pool's other worker, so the
// worker building waits for it and takes the job queued meanwhile, which waits on the lock that worker is filling.
#[test]
fn builds_while_a_worker_of_the_callers_pool_waits_on_it() {
  let pool: ThreadPool = create_pool(2);
  let lock: Arc<PoolOnceLock<u32>> = Arc::new(PoolOnceLock::new());
  let (answer, answers): (Sender<u32>, Receiver<u32>) = mpsc::channel();
  let (second_started, on_second_started): (Sender<()>, Receiver<()>) = mpsc::channel();
  let (second_seen, on_second_seen): (Sender<()>, Receiver<()>) = mpsc::channel();
  let (waiter_started, on_waiter_started): (Sender<()>, Receiver<()>) = mpsc::channel();

  {
    let (lock, answer) = (Arc::clone(&lock), answer.clone());

    pool.spawn(move || {
      let value: u32 = *lock.get_or_init(move || {
        let (first, second): (u32, u32) = rayon::join(
          // Held until the second half runs elsewhere, so a worker with another free beside it has it taken.
          move || {
            let _ = on_second_seen.recv_timeout(TAKEN_WAIT);

            1
          },
          // Held until the waiter runs: on a plain `OnceLock` the building worker is the only one free to run it.
          move || {
            let _ = second_seen.send(());
            let _ = second_started.send(());
            let _ = on_waiter_started.recv_timeout(WAIT);

            2
          },
        );

        first + second
      });

      let _ = answer.send(value);
    });
  }

  on_second_started
    .recv_timeout(WAIT)
    .expect("the build's second half starts");

  {
    let (lock, answer) = (Arc::clone(&lock), answer.clone());

    pool.spawn(move || {
      let _ = waiter_started.send(());
      let _ = answer.send(*lock.get_or_init(|| 0));
    });
  }

  for _ in 0..2 {
    assert_eq!(
      answers.recv_timeout(WAIT),
      Ok(3),
      "every ask answers the one value built"
    );
  }
}

#[test]
fn builds_once_however_many_ask_at_once() {
  let pool: ThreadPool = create_pool(4);
  let lock: PoolOnceLock<u32> = PoolOnceLock::new();
  let builds: AtomicU32 = AtomicU32::new(0);

  assert_eq!(lock.get(), None);

  let values: Vec<u32> = pool.install(|| {
    use rayon::prelude::*;

    (0..16)
      .into_par_iter()
      .map(|_| {
        *lock.get_or_init(|| {
          builds.fetch_add(1, Ordering::Relaxed);

          (0..100u32).into_par_iter().sum()
        })
      })
      .collect()
  });

  assert!(values.iter().all(|value| *value == 4950));
  assert_eq!(builds.load(Ordering::Relaxed), 1);
  assert_eq!(lock.get(), Some(&4950));
}
