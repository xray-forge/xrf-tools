use std::collections::HashMap;

/// Names for a decompiled scene's objects, which the compiled level does not keep: each a prefix and a four-digit
/// number, `<prefix>_0000` upwards, as the Level Editor generates a new object's name (`EScene::GenObjectName`). One
/// instance names a whole scene, so no two objects share a name.
#[derive(Clone, Debug, Default)]
pub struct LevelObjectNames {
  next_numbers: HashMap<String, u32>,
}

impl LevelObjectNames {
  /// The next free name of `prefix`, lowercased as the editor lowercases it.
  pub fn get_next(&mut self, prefix: &str) -> String {
    let prefix: String = prefix.to_lowercase();
    let number: &mut u32 = self.next_numbers.entry(prefix.clone()).or_default();
    let name: String = format!("{prefix}_{number:04}");

    *number += 1;

    name
  }

  /// The next free name after the file a game path names, its last component: `ambient\air_2` gives `air_2_0000`.
  pub fn get_next_after_file(&mut self, path: &str) -> String {
    self.get_next(path.rsplit(['\\', '/']).next().unwrap_or_default())
  }
}
