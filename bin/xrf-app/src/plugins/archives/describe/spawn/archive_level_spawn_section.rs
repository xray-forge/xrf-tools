use std::collections::BTreeMap;

use serde::Serialize;
use xrf_db::LevelSpawnFile;

/// One config section a level spawns objects from, and how many of them.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelSpawnSection {
  /// The config section the objects are built from, which names an LTX section rather than a file.
  pub name: String,
  pub objects: usize,
}

impl ArchiveLevelSpawnSection {
  /// What a level spawns, grouped by section, most planted first.
  pub fn of_all(file: &LevelSpawnFile) -> Vec<Self> {
    let counted: BTreeMap<&str, usize> = file.get_sections();
    let mut sections: Vec<Self> = counted
      .into_iter()
      .map(|(name, objects)| Self {
        name: name.to_owned(),
        objects,
      })
      .collect();

    sections.sort_by(|a, b| b.objects.cmp(&a.objects).then_with(|| a.name.cmp(&b.name)));

    sections
  }
}
