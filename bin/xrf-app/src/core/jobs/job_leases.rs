use std::collections::HashMap;

use uuid::Uuid;

/// What running jobs hold exclusively, and which job holds each of it.
///
/// The registry never interprets a key: what two runs may not do at once is the domain's business, and this only
/// enforces that whatever they named stays held by one of them. Keeping that here rather than as a map inside the
/// registry state is what lets the three rules below be stated once instead of at every site that touches the map.
#[derive(Default)]
pub struct JobLeases {
  held: HashMap<String, Uuid>,
}

impl JobLeases {
  /// The job holding `key`, where one does.
  pub fn get_holder(&self, key: &str) -> Option<Uuid> {
    self.held.get(key).copied()
  }

  /// The first of `keys` somebody already holds.
  ///
  /// Asked before taking any of them, because a registration takes all its keys or none: a job that took the free ones
  /// and failed on the held one would leave them owned by nobody.
  pub fn get_taken<'keys>(&self, keys: &'keys [String]) -> Option<&'keys String> {
    keys.iter().find(|key| self.held.contains_key(*key))
  }

  /// Give every key to `id`.
  ///
  /// Call only where `get_taken` answered `None`. Taking a held key here would overwrite its holder, and the job that
  /// had it would release the successor's lease on its way out.
  pub fn take(&mut self, id: Uuid, keys: &[String]) {
    for key in keys {
      self.held.insert(key.clone(), id);
    }
  }

  /// Release the keys `id` still holds, and only those.
  ///
  /// By holder as well as by key, because releasing by key alone would let a job's teardown hand away a destination a
  /// later job had already taken — worse than never having held a lease at all.
  pub fn release(&mut self, id: Uuid, keys: &[String]) {
    for key in keys {
      if self.get_holder(key) == Some(id) {
        self.held.remove(key);
      }
    }
  }
}
