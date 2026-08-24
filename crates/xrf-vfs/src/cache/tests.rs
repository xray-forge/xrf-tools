use std::sync::Arc;

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
fn serves_a_retained_value_and_counts_the_hit() {
  let cache: XrayAssetCache = cache();
  let scope: XrayLookupScope = scope();

  assert!(cache.get::<Parsed>(&scope, "a.omf").is_none());

  cache.insert(&scope, "a.omf", XrayAssetType::Omf, 64, Arc::new(Parsed("first")));

  assert_eq!(cache.get::<Parsed>(&scope, "a.omf").as_deref(), Some(&Parsed("first")));

  let stats: XrayCacheStats = cache.get_stats();

  assert_eq!(stats.entries, 1);
  assert_eq!(stats.bytes, 64);
  assert_eq!(stats.hits, 1);
  assert_eq!(stats.misses, 1);
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
