use std::cell::Cell;

use xrf_error::XrfError;

use crate::{PipelineAsk, PipelineCache, PipelineMap, PipelineSet};

#[test]
fn makes_a_set_again_once_a_generation_and_keeps_the_last_where_refused() {
  let cache: PipelineCache = PipelineCache::new();
  let mut set: PipelineSet<u32> = PipelineSet::new(&cache, 1);
  let made: Cell<u32> = Cell::new(0);

  assert!(
    set
      .refresh(&cache, || {
        made.set(made.get() + 1);
        Ok(2)
      })
      .is_ok()
  );
  assert_eq!((*set, made.get()), (1, 0), "an unchanged cache makes nothing");

  cache.invalidate_all();

  assert!(
    set
      .refresh(&cache, || Err(XrfError::new_invalid_error("refused")))
      .is_err()
  );
  assert_eq!(*set, 1, "a refused remake keeps the last made");
  assert!(set.refresh(&cache, || Ok(3)).is_ok());
  assert_eq!(*set, 1, "a refused remake is not tried again before the cache changes");

  cache.invalidate_all();

  assert!(set.refresh(&cache, || Ok(4)).is_ok());
  assert_eq!(*set, 4);
}

/// A refused first ask is told once and not made again before the cache changes, so a shader that does not build is not
/// compiled again every frame; then made, and kept while the cache stands.
#[test]
fn asks_for_a_set_once_a_generation_until_one_is_made() {
  let cache: PipelineCache = PipelineCache::new();
  let mut set: PipelineAsk<u32> = PipelineAsk::default();
  let made: Cell<u32> = Cell::new(0);
  let refuse = || {
    made.set(made.get() + 1);
    Err(XrfError::new_invalid_error("refused"))
  };

  assert!(set.ask(&cache, refuse).is_err());
  assert!(set.ask(&cache, refuse).is_ok(), "the refusal is told once");
  assert_eq!(
    (set.get(), made.get()),
    (None, 1),
    "nor tried again before the cache changes"
  );

  cache.invalidate_all();

  assert!(set.ask(&cache, || Ok(1)).is_ok());
  assert!(set.ask(&cache, || Ok(2)).is_ok());
  assert_eq!(set.get().copied(), Some(1), "a made set is kept while the cache stands");
}

/// A key refused on first use answers its refusal again without being made again before the cache changes.
#[test]
fn makes_a_refused_key_again_only_once_the_cache_changes() {
  let cache: PipelineCache = PipelineCache::new();
  let mut map: PipelineMap<u8, u32> = PipelineMap::new(&cache);
  let made: Cell<u32> = Cell::new(0);
  let refuse = |_| {
    made.set(made.get() + 1);
    Err(XrfError::new_invalid_error("refused"))
  };

  assert!(map.prepare(1, refuse).is_err());
  assert!(map.prepare(1, refuse).is_err(), "the refusal stands");
  assert_eq!(
    (map.get(&1), made.get()),
    (None, 1),
    "not made again before the cache changes"
  );

  cache.invalidate_all();

  assert!(map.refresh(&cache, |key| Ok(u32::from(key))).is_ok());
  assert!(map.prepare(1, |key| Ok(u32::from(key) * 10)).is_ok());
  assert_eq!(map.get(&1), Some(&10));
}

#[test]
fn makes_every_key_again_and_keeps_all_where_one_is_refused() {
  let cache: PipelineCache = PipelineCache::new();
  let mut map: PipelineMap<u8, u32> = PipelineMap::new(&cache);

  assert!(map.prepare(1, |key| Ok(u32::from(key) * 10)).is_ok());
  assert!(map.prepare(2, |key| Ok(u32::from(key) * 10)).is_ok());
  assert!(map.prepare(1, |_| Ok(0)).is_ok());
  assert_eq!(map.get(&1), Some(&10), "a made key is not made again");

  cache.invalidate_all();

  assert!(
    map
      .refresh(&cache, |key| match key {
        1 => Ok(11),
        _ => Err(XrfError::new_invalid_error("refused")),
      })
      .is_err()
  );
  assert_eq!(
    (map.get(&1), map.get(&2)),
    (Some(&10), Some(&20)),
    "one refusal keeps every last made"
  );

  cache.invalidate_all();

  assert!(map.refresh(&cache, |key| Ok(u32::from(key) * 100)).is_ok());
  assert_eq!((map.get(&1), map.get(&2)), (Some(&100), Some(&200)));
}
