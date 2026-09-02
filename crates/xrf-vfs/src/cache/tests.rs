use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use xrf_error::{XrfError, XrfResult};

use crate::asset::XrayAssetType;
use crate::cache::{XrayAssetCache, XrayCachePolicy, XrayCacheStats};
use crate::vfs::XrayLookupScope;

/// Stands in for a parsed asset, so the store is exercised without a format crate.
#[derive(Debug, Eq, PartialEq)]
struct Parsed(&'static str);

/// A second type over one path, which is what the type in the key exists to keep apart.
#[derive(Debug, Eq, PartialEq)]
struct Projection(usize);

fn scope() -> XrayLookupScope {
  XrayLookupScope::all()
}

fn cache() -> XrayAssetCache {
  XrayAssetCache::new(XrayCachePolicy::verification())
}

#[test]
fn serves_a_retained_value_without_counting_the_look() {
  let cache: XrayAssetCache = cache();
  let scope: XrayLookupScope = scope();

  assert!(cache.get::<Parsed>(&scope, "a.omf").is_none());

  cache.insert(&scope, "a.omf", XrayAssetType::Omf, 64, Arc::new(Parsed("first")));

  assert_eq!(cache.get::<Parsed>(&scope, "a.omf").as_deref(), Some(&Parsed("first")));

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
      .get_or_load(&scope(), "a.omf", XrayAssetType::Omf, || {
        loads.fetch_add(1, Ordering::Relaxed);

        Ok((Parsed("first"), 64))
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
  let scope: XrayLookupScope = scope();

  // Verification retains motions only, so a visual flows through and is not kept.
  cache.insert(&scope, "a.ogf", XrayAssetType::Ogf, 4096, Arc::new(Parsed("visual")));

  assert!(cache.get::<Parsed>(&scope, "a.ogf").is_none());
  assert_eq!(cache.get_stats().entries, 0);
}

#[test]
fn separates_two_types_over_one_path() {
  let cache: XrayAssetCache = cache();
  let scope: XrayLookupScope = scope();

  cache.insert(&scope, "a.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("whole")));
  cache.insert(&scope, "a.omf", XrayAssetType::Omf, 4, Arc::new(Projection(7)));

  assert_eq!(cache.get::<Parsed>(&scope, "a.omf").as_deref(), Some(&Parsed("whole")));
  assert_eq!(
    cache.get::<Projection>(&scope, "a.omf").as_deref(),
    Some(&Projection(7))
  );
  assert_eq!(cache.get_stats().entries, 2);
}

#[test]
fn separates_two_scopes_over_one_path() {
  let cache: XrayAssetCache = cache();
  let narrowed: XrayLookupScope = XrayLookupScope::all().with_prefix("meshes").unwrap();

  cache.insert(&scope(), "a.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("wide")));

  // A narrowed scope can resolve one path to different bytes, so it must not be served the wide answer.
  assert!(cache.get::<Parsed>(&narrowed, "a.omf").is_none());
  assert_eq!(cache.get::<Parsed>(&scope(), "a.omf").as_deref(), Some(&Parsed("wide")));
}

#[test]
fn drops_only_what_nobody_holds() {
  let cache: XrayAssetCache = cache();
  let scope: XrayLookupScope = scope();

  cache.insert(&scope, "held.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("held")));
  cache.insert(&scope, "free.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("free")));

  let held: Arc<Parsed> = cache.get::<Parsed>(&scope, "held.omf").expect("the entry was inserted");

  assert_eq!(cache.clean_unreferenced(), 1);
  assert!(cache.get::<Parsed>(&scope, "free.omf").is_none());
  assert_eq!(
    cache.get::<Parsed>(&scope, "held.omf").as_deref(),
    Some(&Parsed("held"))
  );

  drop(held);

  assert_eq!(cache.clean_unreferenced(), 1);
  assert_eq!(cache.get_stats().entries, 0);
}

#[test]
fn clears_everything_while_callers_keep_their_own_references() {
  let cache: XrayAssetCache = cache();
  let scope: XrayLookupScope = scope();

  cache.insert(&scope, "a.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("value")));

  let held: Arc<Parsed> = cache.get::<Parsed>(&scope, "a.omf").expect("the entry was inserted");

  cache.clear();

  assert_eq!(cache.get_stats().entries, 0);
  // The caller's value stays valid; only the store let go of it.
  assert_eq!(*held, Parsed("value"));
}

#[test]
fn stops_retaining_at_the_budget_rather_than_exceeding_it() {
  let cache: XrayAssetCache = XrayAssetCache::new(XrayCachePolicy::verification().with_budget(100));
  let scope: XrayLookupScope = scope();

  let held: Arc<Parsed> = Arc::new(Parsed("held"));

  cache.insert(&scope, "held.omf", XrayAssetType::Omf, 80, Arc::clone(&held));
  cache.insert(&scope, "next.omf", XrayAssetType::Omf, 80, Arc::new(Parsed("next")));

  // Nothing was evictable, so the ceiling holds and the second value is simply not kept.
  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.entries, 1);
  assert_eq!(stats.bytes, 80);
  assert_eq!(stats.refused, 1);
  assert!(cache.get::<Parsed>(&scope, "next.omf").is_none());
}

#[test]
fn makes_room_by_dropping_unreferenced_entries_first() {
  let cache: XrayAssetCache = XrayAssetCache::new(XrayCachePolicy::verification().with_budget(100));
  let scope: XrayLookupScope = scope();

  cache.insert(&scope, "free.omf", XrayAssetType::Omf, 80, Arc::new(Parsed("free")));
  cache.insert(&scope, "next.omf", XrayAssetType::Omf, 80, Arc::new(Parsed("next")));

  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.entries, 1);
  assert_eq!(stats.refused, 0);
  assert_eq!(
    cache.get::<Parsed>(&scope, "next.omf").as_deref(),
    Some(&Parsed("next"))
  );
}

#[test]
fn retains_nothing_under_an_empty_policy() {
  let cache: XrayAssetCache = XrayAssetCache::new(XrayCachePolicy::none());

  assert!(XrayCachePolicy::none().is_empty());

  cache.insert(&scope(), "a.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("value")));

  assert_eq!(cache.get_stats().entries, 0);
}

#[test]
fn forgets_one_path_under_every_type_and_scope() {
  let cache: XrayAssetCache = cache();
  let narrowed: XrayLookupScope = XrayLookupScope::all().with_prefix("meshes").unwrap();

  cache.insert(&scope(), "a.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("whole")));
  cache.insert(&scope(), "a.omf", XrayAssetType::Omf, 4, Arc::new(Projection(1)));
  cache.insert(&narrowed, "a.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("narrow")));
  cache.insert(&scope(), "b.omf", XrayAssetType::Omf, 8, Arc::new(Parsed("other")));

  // A write changes that path's bytes for anyone: every shape and every scope holding it has to go, because working out
  // which scopes were serving the written mount needs a resolve each.
  assert_eq!(cache.forget("a.omf"), 3);
  assert!(cache.get::<Parsed>(&scope(), "a.omf").is_none());
  assert!(cache.get::<Projection>(&scope(), "a.omf").is_none());
  assert!(cache.get::<Parsed>(&narrowed, "a.omf").is_none());
  assert_eq!(
    cache.get::<Parsed>(&scope(), "b.omf").as_deref(),
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
  F: Fn() -> XrfResult<(T, u64)> + Send + Sync,
{
  let start: Barrier = Barrier::new(2);

  thread::scope(|threads| {
    let handles: Vec<_> = (0..2)
      .map(|_| {
        threads.spawn(|| {
          start.wait();

          cache.get_or_load(&scope(), "shared.omf", XrayAssetType::Omf, &load)
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

    Ok((Parsed("shared"), 64))
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
          .get_or_load(&scope(), "shared.ogf", XrayAssetType::Ogf, || {
            loads.fetch_add(1, Ordering::Relaxed);
            thread::sleep(Duration::from_millis(50));

            Ok((Parsed("visual"), 4096))
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
        cache.get_or_load::<Parsed, _>(&scope(), "shared.omf", XrayAssetType::Omf, || {
          start.wait();
          thread::sleep(Duration::from_millis(50));

          panic!("the parse gave up");
        })
      }))
    });

    let waiter = threads.spawn(|| {
      start.wait();

      cache.get_or_load(&scope(), "shared.omf", XrayAssetType::Omf, || {
        Ok((Parsed("second try"), 64))
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
