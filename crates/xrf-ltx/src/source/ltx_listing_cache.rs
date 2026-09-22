use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use xrf_error::XrfResult;

/// File names per directory, listed once for the life of a project.
#[derive(Debug, Default)]
pub(crate) struct LtxListingCache {
  listings: Mutex<HashMap<String, Arc<[String]>>>,
}

impl LtxListingCache {
  /// The names in `directory`, from `list` on the first request.
  pub fn get_or_list(
    &self,
    directory: &str,
    list: impl FnOnce() -> XrfResult<Vec<String>>,
  ) -> XrfResult<Arc<[String]>> {
    if let Some(names) = self.lock().get(directory) {
      return Ok(Arc::clone(names));
    }

    let names: Arc<[String]> = Arc::from(list()?);

    self.lock().insert(String::from(directory), Arc::clone(&names));

    Ok(names)
  }

  fn lock(&self) -> MutexGuard<'_, HashMap<String, Arc<[String]>>> {
    self.listings.lock().expect("listing cache to not be poisoned")
  }
}

#[cfg(test)]
mod tests {
  use std::cell::Cell;
  use std::sync::Arc;

  use xrf_error::XrfError;

  use super::LtxListingCache;

  #[test]
  fn lists_each_directory_once() {
    let cache: LtxListingCache = LtxListingCache::default();
    let listed: Cell<u32> = Cell::new(0);
    let list = || {
      listed.set(listed.get() + 1);

      Ok(vec![String::from("system.ltx")])
    };

    let first: Arc<[String]> = cache.get_or_list("configs", list).unwrap();
    let second: Arc<[String]> = cache.get_or_list("configs", list).unwrap();

    assert_eq!(listed.get(), 1);
    assert!(Arc::ptr_eq(&first, &second));

    cache.get_or_list("configs\\items", list).unwrap();

    assert_eq!(listed.get(), 2);
  }

  #[test]
  fn does_not_remember_a_failed_listing() {
    let cache: LtxListingCache = LtxListingCache::default();

    assert!(
      cache
        .get_or_list("configs", || Err(XrfError::new_unexpected_error("unreadable")))
        .is_err()
    );
    assert_eq!(
      &*cache.get_or_list("configs", || Ok(Vec::new())).unwrap(),
      &[] as &[String]
    );
  }
}
