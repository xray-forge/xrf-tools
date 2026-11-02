use xrf_vfs::XrayLogicalPath;

use crate::level::LevelSource;

#[test]
fn names_an_asset_level_by_the_path_it_was_opened_as() {
  assert_eq!(
    LevelSource::Asset {
      logical_path: String::from("levels\\k00_marsh"),
    }
    .get_logical_directory()
    .as_ref()
    .map(XrayLogicalPath::as_str),
    Some("levels\\k00_marsh")
  );
}

#[test]
fn names_a_directory_level_by_its_own_name_below_levels() {
  // A loose directory still has an engine identity, which is what its lightmaps are addressed under.
  assert_eq!(
    LevelSource::Directory {
      path: String::from("C:\\game\\gamedata\\levels\\l01_escape"),
    }
    .get_logical_directory()
    .as_ref()
    .map(XrayLogicalPath::as_str),
    Some("levels\\l01_escape")
  );
}

#[test]
fn names_a_level_by_the_last_component_of_its_identity_in_lower_case() {
  assert_eq!(
    LevelSource::Asset {
      logical_path: String::from("levels\\K00_Marsh"),
    }
    .get_name()
    .as_deref(),
    Some("k00_marsh")
  );
}

#[test]
fn names_nothing_for_a_directory_with_no_final_component() {
  assert_eq!(
    LevelSource::Directory { path: String::from("") }.get_logical_directory(),
    None
  );
}
