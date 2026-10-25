use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_app_core::session::SessionSnapshot;
use xrf_renderer::{RenderLevelSpawn, RenderSpawnObject};
use xrf_spawn::AlifeObject;
use xrf_vfs::XraySearch;
use xrf_visual::VisualTransform;

use crate::level::SelectedLevel;
use crate::parts::{LevelSpawn, LevelSpawnPart};
use crate::report::LevelReport;
use crate::spawn::{LevelSpawnCategory, LevelSpawnObject};

/// The spawned objects the viewer draws, and the visuals they stand as, each named once.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSpawnObjectsDescription {
  pub visuals: Vec<String>,
  pub objects: Vec<LevelSpawnObject>,
}

impl LevelSpawnObjectsDescription {
  /// The objects the open level's spawn draws, read through `search`, with how long that took.
  ///
  /// # Errors
  ///
  /// Returns an error when the spawn cannot be read.
  pub fn read(current: &SessionSnapshot<SelectedLevel>, search: &XraySearch) -> TauriResult<Self> {
    let started: Instant = Instant::now();
    let spawn: Arc<LevelSpawn> = LevelSpawnPart::get(current, search)
      .inspect_err(|error| LevelReport::missing_spawn_objects(&current.source, error))?;
    let described: Self = Self::describe(&spawn);

    LevelReport::spawn_objects(&current.source, &described, started);

    Ok(described)
  }

  /// Every object of a level's spawn the viewer draws, in the order the spawn keeps them, each visual named once.
  pub fn describe(spawn: &LevelSpawn) -> Self {
    let mut objects: Vec<LevelSpawnObject> = Vec::new();
    let visuals: Vec<String> = Self::fold(spawn, |index, object, category, visual| {
      objects.push(LevelSpawnObject {
        category,
        clsid: object.clsid.clone(),
        index,
        name: object.name.clone(),
        release: spawn.get_release(index as usize),
        section: object.section.clone(),
        story_id: object.inherited.get_abstract().and_then(|it| it.get_story_id()),
        transform: VisualTransform::of_spawn(&object.position, &object.direction),
        visual,
      });
    });

    Self { visuals, objects }
  }

  /// The drawn objects as the renderer stands them: what `describe` answers without any name, section or class the
  /// renderer has no use for.
  pub fn to_render_spawn(spawn: &LevelSpawn) -> RenderLevelSpawn {
    let mut objects: Vec<RenderSpawnObject> = Vec::new();
    let visuals: Vec<String> = Self::fold(spawn, |index, object, category, visual| {
      objects.push(RenderSpawnObject {
        index,
        category: category.into(),
        is_released: spawn.get_release(index as usize).is_some(),
        visual,
        transform: VisualTransform::of_spawn(&object.position, &object.direction).to_matrix(),
      });
    });

    RenderLevelSpawn {
      objects,
      visuals,
      detail: 0.0,
    }
  }

  /// Hands `each` every drawn object of the spawn with its place, category and visual index, answering the visuals
  /// those index, each named once in the order first stood.
  fn fold(spawn: &LevelSpawn, mut each: impl FnMut(u32, &AlifeObject, LevelSpawnCategory, u32)) -> Vec<String> {
    let mut visuals: Vec<String> = Vec::new();
    let mut indices: HashMap<&str, u32> = HashMap::new();

    for (index, object) in spawn.objects.iter().enumerate() {
      let Some((category, name)) = LevelSpawnCategory::get_drawn_visual(object) else {
        continue;
      };

      let visual: u32 = *indices.entry(name).or_insert_with(|| {
        visuals.push(name.to_owned());

        (visuals.len() - 1) as u32
      });

      each(index as u32, object, category, visual);
    }

    visuals
  }
}
