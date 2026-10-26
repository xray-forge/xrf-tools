use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use xrf_error::{XrfError, XrfResult};

use crate::asset::XrayAssetType;
use crate::vfs::cache::{XrayAssetCache, XrayCacheKey, XrayCachePolicy, XrayCacheStats};
use crate::{XrayFileStamp, XrayRevision};

/// Stands in for a parsed asset, so the store is exercised without a format crate.
#[derive(Debug, Eq, PartialEq)]
struct Parsed(&'static str);

/// A second type over one path, which is what the type in the key exists to keep apart.
#[derive(Debug, Eq, PartialEq)]
struct Projection(usize);

/// The opening every key here names unless a test is about two of them.
const SOURCE: u64 = 0;

/// One copy of `path` in the default source, as it was mounted.
fn key<T: 'static>(path: &str) -> XrayCacheKey {
  XrayCacheKey::of::<T>(SOURCE, path, XrayRevision::MOUNTED)
}

fn cache() -> XrayAssetCache {
  XrayAssetCache::new(XrayCachePolicy::verification())
}

#[test]
fn serves_a_retained_value_without_counting_the_look() {
  let cache: XrayAssetCache = cache();
  assert!(cache.get::<Parsed>(&key::<Parsed>("a.omf")).is_none());

  cache.insert(
    key::<Parsed>("a.omf"),
    XrayAssetType::Omf,
    64,
    Arc::new(Parsed("first")),
  );

  assert_eq!(
    cache.get::<Parsed>(&key::<Parsed>("a.omf")).as_deref(),
    Some(&Parsed("first"))
  );

  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.entries, 1);
  assert_eq!(stats.bytes, 64);
  // Inspecting the store is not a read a run made, so looking cannot change what the run reports about itself.
  assert_eq!(stats.hits, 0);
  assert_eq!(stats.misses, 0);
}

/// The sequential baseline the coordinated race is measured against.
///
/// Two reads of one key are one miss and one hit. `loads_a_retained_key_once_for_two_concurrent_misses` asserts the
/// same pair for two readers arriving together, and the two tests are only meaningful beside each other: together they
/// say the counters describe the inputs rather than the schedule.
#[test]
fn counts_one_miss_then_one_hit_for_two_reads_of_one_key() {
  let cache: XrayAssetCache = cache();
  let loads: AtomicUsize = AtomicUsize::new(0);

  for _ in 0..2 {
    cache
      .get_or_load(key::<Parsed>("a.omf"), XrayAssetType::Omf, || {
        loads.fetch_add(1, Ordering::Relaxed);

        Ok((Parsed("first"), 64, None))
      })
      .expect("both reads are served");
  }

  assert_eq!(loads.load(Ordering::Relaxed), 1);

  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.misses, 1);
  assert_eq!(stats.hits, 1);
  assert_eq!(stats.entries, 1);
  assert_eq!(stats.bytes, 64);
}

#[test]
fn keeps_kinds_the_policy_excludes_out() {
  let cache: XrayAssetCache = cache();
  // Verification retains motions only, so a visual flows through and is not kept.
  cache.insert(
    key::<Parsed>("a.ogf"),
    XrayAssetType::Ogf,
    4096,
    Arc::new(Parsed("visual")),
  );

  assert!(cache.get::<Parsed>(&key::<Parsed>("a.ogf")).is_none());
  assert_eq!(cache.get_stats().entries, 0);
}

#[test]
fn separates_two_types_over_one_path() {
  let cache: XrayAssetCache = cache();
  cache.insert(key::<Parsed>("a.omf"), XrayAssetType::Omf, 8, Arc::new(Parsed("whole")));
  cache.insert(
    key::<Projection>("a.omf"),
    XrayAssetType::Omf,
    4,
    Arc::new(Projection(7)),
  );

  assert_eq!(
    cache.get::<Parsed>(&key::<Parsed>("a.omf")).as_deref(),
    Some(&Parsed("whole"))
  );
  assert_eq!(
    cache.get::<Projection>(&key::<Projection>("a.omf")).as_deref(),
    Some(&Projection(7))
  );
  assert_eq!(cache.get_stats().entries, 2);
}

#[test]
fn separates_two_copies_and_two_revisions_of_one_path() {
  let cache: XrayAssetCache = cache();
  let other_source: XrayCacheKey = XrayCacheKey::of::<Parsed>(SOURCE + 1, "a.omf", XrayRevision::MOUNTED);

  cache.insert(key::<Parsed>("a.omf"), XrayAssetType::Omf, 8, Arc::new(Parsed("game")));

  // Another source's copy of the path holds other bytes, and so does this copy once a change moved its revision.
  assert!(cache.get::<Parsed>(&other_source).is_none());
  assert!(
    cache
      .get::<Parsed>(&XrayCacheKey::of::<Parsed>(
        SOURCE,
        "a.omf",
        XrayRevision::from_count(1)
      ))
      .is_none()
  );
  assert_eq!(
    cache.get::<Parsed>(&key::<Parsed>("a.omf")).as_deref(),
    Some(&Parsed("game"))
  );
}

#[test]
fn drops_only_what_nobody_holds() {
  let cache: XrayAssetCache = cache();
  cache.insert(
    key::<Parsed>("held.omf"),
    XrayAssetType::Omf,
    8,
    Arc::new(Parsed("held")),
  );
  cache.insert(
    key::<Parsed>("free.omf"),
    XrayAssetType::Omf,
    8,
    Arc::new(Parsed("free")),
  );

  let held: Arc<Parsed> = cache
    .get::<Parsed>(&key::<Parsed>("held.omf"))
    .expect("the entry was inserted");

  assert_eq!(cache.clean_unreferenced(), 1);
  assert!(cache.get::<Parsed>(&key::<Parsed>("free.omf")).is_none());
  assert_eq!(
    cache.get::<Parsed>(&key::<Parsed>("held.omf")).as_deref(),
    Some(&Parsed("held"))
  );

  drop(held);

  assert_eq!(cache.clean_unreferenced(), 1);
  assert_eq!(cache.get_stats().entries, 0);
}

// At the ceiling an insert drops the oldest values nobody holds, only as many as it needs, rather than the whole warm
// set.
#[test]
fn makes_room_by_dropping_the_oldest_unheld_entries_alone() {
  let cache: XrayAssetCache = XrayAssetCache::new(XrayCachePolicy::verification().with_budget(100));

  for name in ["oldest.omf", "older.omf", "newer.omf"] {
    cache.insert(key::<Parsed>(name), XrayAssetType::Omf, 30, Arc::new(Parsed("value")));
  }

  cache.insert(
    key::<Parsed>("next.omf"),
    XrayAssetType::Omf,
    40,
    Arc::new(Parsed("next")),
  );

  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!((stats.entries, stats.bytes, stats.refused), (3, 100, 0));
  assert!(cache.get::<Parsed>(&key::<Parsed>("oldest.omf")).is_none());
  assert!(cache.get::<Parsed>(&key::<Parsed>("older.omf")).is_some());
  assert!(cache.get::<Parsed>(&key::<Parsed>("newer.omf")).is_some());
}

#[test]
fn stops_retaining_at_the_budget_rather_than_exceeding_it() {
  let cache: XrayAssetCache = XrayAssetCache::new(XrayCachePolicy::verification().with_budget(100));
  let held: Arc<Parsed> = Arc::new(Parsed("held"));

  cache.insert(key::<Parsed>("held.omf"), XrayAssetType::Omf, 80, Arc::clone(&held));
  cache.insert(
    key::<Parsed>("next.omf"),
    XrayAssetType::Omf,
    80,
    Arc::new(Parsed("next")),
  );

  // Nothing was evictable, so the ceiling holds and the second value is simply not kept.
  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.entries, 1);
  assert_eq!(stats.bytes, 80);
  assert_eq!(stats.refused, 1);
  assert!(cache.get::<Parsed>(&key::<Parsed>("next.omf")).is_none());
}

#[test]
fn makes_room_by_dropping_unreferenced_entries_first() {
  let cache: XrayAssetCache = XrayAssetCache::new(XrayCachePolicy::verification().with_budget(100));
  cache.insert(
    key::<Parsed>("free.omf"),
    XrayAssetType::Omf,
    80,
    Arc::new(Parsed("free")),
  );
  cache.insert(
    key::<Parsed>("next.omf"),
    XrayAssetType::Omf,
    80,
    Arc::new(Parsed("next")),
  );

  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.entries, 1);
  assert_eq!(stats.refused, 0);
  assert_eq!(
    cache.get::<Parsed>(&key::<Parsed>("next.omf")).as_deref(),
    Some(&Parsed("next"))
  );
}

#[test]
fn retains_nothing_under_an_empty_policy() {
  let cache: XrayAssetCache = XrayAssetCache::new(XrayCachePolicy::none());

  assert!(XrayCachePolicy::none().is_empty());

  cache.insert(key::<Parsed>("a.omf"), XrayAssetType::Omf, 8, Arc::new(Parsed("value")));

  assert_eq!(cache.get_stats().entries, 0);
}

#[test]
fn forgets_one_copy_under_every_type_and_revision() {
  let cache: XrayAssetCache = cache();
  let other_source: XrayCacheKey = XrayCacheKey::of::<Parsed>(SOURCE + 1, "a.omf", XrayRevision::MOUNTED);

  cache.insert(key::<Parsed>("a.omf"), XrayAssetType::Omf, 8, Arc::new(Parsed("whole")));
  cache.insert(
    key::<Projection>("a.omf"),
    XrayAssetType::Omf,
    4,
    Arc::new(Projection(1)),
  );
  cache.insert(
    XrayCacheKey::of::<Parsed>(SOURCE, "a.omf", XrayRevision::from_count(1)),
    XrayAssetType::Omf,
    8,
    Arc::new(Parsed("changed")),
  );
  cache.insert(other_source.clone(), XrayAssetType::Omf, 8, Arc::new(Parsed("mod")));
  cache.insert(key::<Parsed>("b.omf"), XrayAssetType::Omf, 8, Arc::new(Parsed("other")));

  // A write changes that copy's bytes for anyone: every shape of it goes, and another source's copy of the path stays.
  assert_eq!(cache.forget_copy(SOURCE, "a.omf"), 3);
  assert!(cache.get::<Parsed>(&key::<Parsed>("a.omf")).is_none());
  assert!(cache.get::<Projection>(&key::<Projection>("a.omf")).is_none());
  assert_eq!(cache.get::<Parsed>(&other_source).as_deref(), Some(&Parsed("mod")));
  assert_eq!(
    cache.get::<Parsed>(&key::<Parsed>("b.omf")).as_deref(),
    Some(&Parsed("other"))
  );
}

/// Runs `load` on two threads released together, so both are inside the store before either can finish.
///
/// A barrier rather than a repeat count: the race this coordinates is two threads missing one key at the same instant,
/// and a test that only sometimes produces it would pass for the same reason the defect it guards survived to now.
fn race<T, F>(cache: &XrayAssetCache, load: F) -> Vec<XrfResult<Arc<T>>>
where
  T: Send + Sync + 'static,
  F: Fn() -> XrfResult<(T, u64, Option<XrayFileStamp>)> + Send + Sync,
{
  let start: Barrier = Barrier::new(2);

  thread::scope(|threads| {
    let handles: Vec<_> = (0..2)
      .map(|_| {
        threads.spawn(|| {
          start.wait();

          cache.get_or_load(key::<Parsed>("shared.omf"), XrayAssetType::Omf, &load)
        })
      })
      .collect();

    handles
      .into_iter()
      .map(|handle| handle.join().expect("worker threads do not panic"))
      .collect()
  })
}

#[test]
fn loads_a_retained_key_once_for_two_concurrent_misses() {
  let cache: XrayAssetCache = cache();
  let loads: AtomicUsize = AtomicUsize::new(0);

  let results: Vec<XrfResult<Arc<Parsed>>> = race(&cache, || {
    loads.fetch_add(1, Ordering::Relaxed);
    // Long enough that the second thread is certainly waiting rather than arriving after the fact.
    thread::sleep(Duration::from_millis(50));

    Ok((Parsed("shared"), 64, None))
  });

  assert_eq!(
    loads.load(Ordering::Relaxed),
    1,
    "the second requester repeated the load"
  );

  let values: Vec<Arc<Parsed>> = results
    .into_iter()
    .map(|result| result.expect("both requesters are served"))
    .collect();

  // One load means one value: both requesters hold the same allocation, not equal copies of it.
  assert!(Arc::ptr_eq(&values[0], &values[1]));

  // What the sequential pair would have reported, which is the whole point: the split does not move with the schedule.
  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.misses, 1);
  assert_eq!(stats.hits, 1);
  assert_eq!(stats.entries, 1);
}

#[test]
fn lets_every_requester_of_an_excluded_kind_load_for_itself() {
  let cache: XrayAssetCache = cache();
  let loads: AtomicUsize = AtomicUsize::new(0);
  let start: Barrier = Barrier::new(2);

  // Verification retains motions only, so nothing would be published for a waiter to find. Coordinating these would
  // turn the second read's miss into a hit and make the counters depend on the schedule, which is the opposite of what
  // the coordination is for.
  thread::scope(|threads| {
    for _ in 0..2 {
      threads.spawn(|| {
        start.wait();

        cache
          .get_or_load(key::<Parsed>("shared.ogf"), XrayAssetType::Ogf, || {
            loads.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(50));

            Ok((Parsed("visual"), 4096, None))
          })
          .expect("both requesters are served");
      });
    }
  });

  assert_eq!(loads.load(Ordering::Relaxed), 2);

  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.misses, 2);
  assert_eq!(stats.hits, 0);
  assert_eq!(stats.entries, 0);
}

#[test]
fn gives_a_waiter_its_own_failure_rather_than_a_copy_of_the_owners() {
  let cache: XrayAssetCache = cache();
  let loads: AtomicUsize = AtomicUsize::new(0);

  let results: Vec<XrfResult<Arc<Parsed>>> = race(&cache, || {
    loads.fetch_add(1, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(50));

    Err(XrfError::new_read_error("shared.omf is not readable"))
  });

  // A failure is not published, so the waiter finds nothing and reads for itself, exactly as the second of two
  // sequential reads would have. Both report the real error rather than one being handed a copy of the other's.
  assert_eq!(loads.load(Ordering::Relaxed), 2);
  assert!(results.iter().all(XrfResult::is_err));

  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.misses, 2);
  assert_eq!(stats.hits, 0);
  assert_eq!(stats.entries, 0);
}

#[test]
fn wakes_waiters_when_the_owners_load_panics() {
  let cache: XrayAssetCache = cache();
  let start: Barrier = Barrier::new(2);

  // Without the flight guard this hangs for the life of the process, which is the one failure of coordinated loading a
  // caller could not diagnose from anything it can see.
  thread::scope(|threads| {
    let owner = threads.spawn(|| {
      catch_unwind(AssertUnwindSafe(|| {
        cache.get_or_load::<Parsed, _>(key::<Parsed>("shared.omf"), XrayAssetType::Omf, || {
          start.wait();
          thread::sleep(Duration::from_millis(50));

          panic!("the parse gave up");
        })
      }))
    });

    let waiter = threads.spawn(|| {
      start.wait();

      cache.get_or_load(key::<Parsed>("shared.omf"), XrayAssetType::Omf, || {
        Ok((Parsed("second try"), 64, None))
      })
    });

    assert!(owner.join().expect("the panic is caught inside the thread").is_err());
    assert_eq!(
      waiter
        .join()
        .expect("the waiter is released rather than stranded")
        .expect("the waiter loads for itself")
        .as_ref(),
      &Parsed("second try")
    );
  });
}
