use serde::{Deserialize, Serialize};
use xrf_renderer::RenderSpawnCategory;

use crate::level::streaming::hidden_spawn_groups::HiddenSpawnGroups;
use crate::level::streaming::spawn_filter::SpawnFilter;

/// What of the level's world plays, rather than what is drawn of it: the weather's rain, bolts and wind, the campfires,
/// the ambient effects, and which groups of the spawned objects stream in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldToggles {
  /// Whether the weather's rain falls and wets surfaces.
  pub is_rainy: bool,
  /// Whether the weather's bolts strike.
  pub is_thundering: bool,
  /// Whether the weather's wind sways trees and grass.
  pub is_windy: bool,
  /// Whether its campfires burn, as `CZoneCampfire` starts, rather than smoulder out.
  pub is_campfire_lit: bool,
  /// Whether the weather's ambient effects play near the camera and bring their wind.
  pub is_ambient_played: bool,
  /// Which groups of the level's spawned objects are drawn.
  pub is_spawned_props: bool,
  pub is_spawned_items: bool,
  pub is_spawned_weapons: bool,
  pub is_spawned_lamps: bool,
  /// Whether the spawned objects a new game releases are drawn too, each with its group.
  pub is_spawned_released: bool,
  /// The spawned objects left out whatever their group, by their index among the level's spawned objects, ascending:
  /// those the viewer hides one by one.
  pub hidden_spawn_objects: Vec<u32>,
}

impl Default for WorldToggles {
  fn default() -> Self {
    Self {
      is_rainy: true,
      is_thundering: true,
      is_windy: true,
      is_campfire_lit: true,
      is_ambient_played: true,
      is_spawned_props: true,
      is_spawned_items: true,
      is_spawned_weapons: true,
      is_spawned_lamps: true,
      is_spawned_released: false,
      hidden_spawn_objects: Vec::new(),
    }
  }
}

impl WorldToggles {
  /// Whether a group of spawned objects is drawn.
  pub fn is_spawned(&self, category: RenderSpawnCategory) -> bool {
    match category {
      RenderSpawnCategory::Props => self.is_spawned_props,
      RenderSpawnCategory::Items => self.is_spawned_items,
      RenderSpawnCategory::Weapons => self.is_spawned_weapons,
      RenderSpawnCategory::Lamps => self.is_spawned_lamps,
    }
  }

  /// What of the spawn is left out of the scene: the groups, and the objects hidden one by one.
  pub(crate) fn get_spawn_filter(&self) -> SpawnFilter<'_> {
    SpawnFilter {
      groups: self.get_hidden_spawn_groups(),
      objects: &self.hidden_spawn_objects,
    }
  }

  /// The spawn groups left out of the scene: each category not drawn, and the released objects of every one while no
  /// released objects are drawn.
  pub(crate) fn get_hidden_spawn_groups(&self) -> HiddenSpawnGroups {
    RenderSpawnCategory::ALL
      .into_iter()
      .fold(HiddenSpawnGroups::default(), |hidden, category| {
        let is_shown: bool = self.is_spawned(category);
        let hidden: HiddenSpawnGroups = if is_shown {
          hidden
        } else {
          hidden.with(category.get_group())
        };

        if is_shown && self.is_spawned_released {
          hidden
        } else {
          hidden.with(category.get_released_group())
        }
      })
  }
}
