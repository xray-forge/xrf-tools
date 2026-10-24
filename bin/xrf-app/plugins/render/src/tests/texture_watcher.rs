use std::path::Path;

use crate::texture_watcher::to_texture_name;

#[test]
fn names_a_written_texture_by_its_file_name_lowercase() {
  assert_eq!(
    to_texture_name(Path::new("D:/stalker/gamedata/textures/ston/Ston_01.DDS")),
    Some(String::from("ston_01"))
  );
  assert_eq!(
    to_texture_name(Path::new("D:/stalker/gamedata/textures/ston/ston_01_bump#.dds")),
    Some(String::from("ston_01_bump#"))
  );
}

#[test]
fn names_nothing_for_a_file_that_is_no_texture() {
  assert_eq!(
    to_texture_name(Path::new("D:/stalker/gamedata/textures/ston/ston_01.thm")),
    None
  );
  assert_eq!(to_texture_name(Path::new("D:/stalker/gamedata/textures/ston")), None);
}
