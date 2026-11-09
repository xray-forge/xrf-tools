use std::collections::{HashMap, HashSet};

/// Names for a decompiled scene's objects, which the compiled level does not keep: each a prefix and a four-digit
/// number, `<prefix>_0000` upwards, as the Level Editor generates a new object's name (`EScene::GenObjectName`). One
/// instance names a whole scene, so no two objects share a name.
#[derive(Clone, Debug, Default)]
pub struct LevelObjectNames {
  next_numbers: HashMap<String, u32>,
  /// Names the level itself keeps, such as an entity's, which no generated name may take.
  reserved: HashSet<String>,
}

impl LevelObjectNames {
  /// Keeps `name` from every name generated after, for an object that comes back under a name of its own.
  pub fn reserve(&mut self, name: &str) {
    self.reserved.insert(name.to_lowercase());
  }

  /// The next free name of `prefix`, lowercased as the editor lowercases it, past any name reserved.
  pub fn get_next(&mut self, prefix: &str) -> String {
    let prefix: String = prefix.to_lowercase();
    let number: &mut u32 = self.next_numbers.entry(prefix.clone()).or_default();

    loop {
      let name: String = format!("{prefix}_{number:04}");

      *number += 1;

      if !self.reserved.contains(&name) {
        return name;
      }
    }
  }

  /// The next free name after the file a game path names, its last component: `ambient\air_2` gives `air_2_0000`.
  pub fn get_next_after_file(&mut self, path: &str) -> String {
    self.get_next(path.rsplit(['\\', '/']).next().unwrap_or_default())
  }
}
