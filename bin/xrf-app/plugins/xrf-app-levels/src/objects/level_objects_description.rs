use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionSnapshot;
use xrf_vfs::XraySearch;
use xrf_visual::VisualTransform;

use crate::level::SelectedLevel;
use crate::objects::{LevelObject, LevelObjectId, LevelObjectKind, LevelObjectSpawn, LevelObjectVolume};
use crate::parts::{LevelSpawn, LevelSpawnPart};
use crate::report::LevelReport;

/// Every level object of a level, in the order its sources keep them, and the visuals they are drawn as, each named
/// once.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectsDescription {
  pub visuals: Vec<String>,
  pub objects: Vec<LevelObject>,
}

impl LevelObjectsDescription {
  /// The open level's objects, read through `search`, with how long that took.
  ///
  /// # Errors
  ///
  /// Returns an error when the spawn cannot be read.
  pub fn read(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> TauriResult<Self> {
    let started: Instant = Instant::now();
    let spawn: Arc<LevelSpawn> = LevelSpawnPart::get(current, search)
      .inspect_err(|error| LevelReport::missing_level_objects(&current.source, error))?;
    let described: Self = Self::describe(&spawn);

    LevelReport::level_objects(&current.source, &described, started);

    Ok(described)
  }

  /// Every object of a level's spawn the level holds, in the spawn's order, each visual named once.
  pub fn describe(spawn: &LevelSpawn) -> Self {
    let mut visuals: Vec<String> = Vec::new();
    let mut indices: HashMap<&str, u32> = HashMap::new();
    let mut objects: Vec<LevelObject> = Vec::new();

    for (index, object) in spawn.objects.iter().enumerate() {
      let Some(kind) = LevelObjectKind::of(object) else {
        continue;
      };
      let visual: Option<u32> = kind
        .get_entry()
        .is_drawn
        .then(|| object.inherited.get_visual())
        .flatten()
        .map(|name| {
          *indices.entry(name).or_insert_with(|| {
            visuals.push(name.to_owned());

            (visuals.len() - 1) as u32
          })
        });

      objects.push(LevelObject {
        id: LevelObjectId::Spawn { index: index as u32 },
        kind,
        name: object.name.clone(),
        transform: VisualTransform::of_spawn(&object.position, &object.direction),
        spawn: Some(LevelObjectSpawn {
          section: object.section.clone(),
          clsid: object.clsid.clone(),
          story_id: object.inherited.get_abstract().and_then(|it| it.get_story_id()),
          release: spawn.get_release(index),
          visual,
        }),
        volume: LevelObjectVolume::of(&object.inherited),
      });
    }

    Self { visuals, objects }
  }
}
