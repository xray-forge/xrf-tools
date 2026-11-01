//! Holds which spawn groups the toggles leave out of a scene, and that the objects of no group always stand.

use xrf_renderer::RenderSpawnCategory;

use crate::contract::world_toggles::WorldToggles;
use crate::level::streaming::hidden_spawn_groups::HiddenSpawnGroups;

#[test]
fn stands_the_objects_of_no_group_whatever_is_hidden() {
  let hidden: HiddenSpawnGroups = (0..=8).fold(HiddenSpawnGroups::default(), HiddenSpawnGroups::with);

  assert!(hidden.is_shown(0));
  assert!(!hidden.is_shown(1));
  assert!(!hidden.is_shown(8));
}

#[test]
fn hides_only_the_released_objects_by_default() {
  let hidden: HiddenSpawnGroups = WorldToggles::default().get_hidden_spawn_groups();

  for category in RenderSpawnCategory::ALL {
    assert!(hidden.is_shown(category.get_group()), "{category:?} kept");
    assert!(!hidden.is_shown(category.get_released_group()), "{category:?} released");
  }
}

#[test]
fn leaves_out_an_object_hidden_one_by_one_whatever_its_group_shows() {
  let toggles: WorldToggles = WorldToggles {
    hidden_spawn_objects: vec![3, 9],
    ..WorldToggles::default()
  };
  let filter = toggles.get_spawn_filter();
  let props: u32 = RenderSpawnCategory::Props.get_group();

  assert!(!filter.is_shown(props, 9));
  assert!(filter.is_shown(props, 4));
  assert!(!filter.is_shown(RenderSpawnCategory::Props.get_released_group(), 4));
}

#[test]
fn hides_a_category_switched_off_with_its_released_objects() {
  let toggles: WorldToggles = WorldToggles {
    is_spawned_weapons: false,
    is_spawned_released: true,
    ..WorldToggles::default()
  };
  let hidden: HiddenSpawnGroups = toggles.get_hidden_spawn_groups();

  assert!(!hidden.is_shown(RenderSpawnCategory::Weapons.get_group()));
  assert!(!hidden.is_shown(RenderSpawnCategory::Weapons.get_released_group()));
  assert!(hidden.is_shown(RenderSpawnCategory::Lamps.get_group()));
  assert!(hidden.is_shown(RenderSpawnCategory::Lamps.get_released_group()));
}
