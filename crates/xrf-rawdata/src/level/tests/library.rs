//! The object library a decompile lists once for every step that looks in it.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_chunk::XRayByteOrder;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_math::Vector3d;
use xrf_test_utils::{GeneratedResources, HostFilesystem};

use crate::level::level_object_library::LevelObjectLibrary;
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::{ObjectDecompiler, SurfaceDress};

/// One triangle's object, written at `path`.
fn write_object(path: &Path) -> XrfResult {
  let floor = [[
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(0.0, 0.0, 1.0),
    Vector3d::new(1.0, 0.0, 0.0),
  ]];

  fs::create_dir_all(path.parent().expect("a folder"))?;
  ObjectDecompiler::decompile_level_geometry(&LevelVisualGeometry::of_triangles(
    "floor\\floor_tile",
    "default",
    &SurfaceDress::default(),
    &floor,
  ))?
  .object
  .write_to_path::<XRayByteOrder, _>(&path)
}

fn references(library: &LevelObjectLibrary) -> Vec<&str> {
  library
    .get_entries()
    .iter()
    .map(|entry| entry.reference.as_str())
    .collect()
}

#[test]
fn lists_each_roots_objects_in_name_order_through_their_folders() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "order"))?;

  for name in ["b.object", "a\\z.OBJECT", "a\\b\\c.object", "detail\\grass.object"] {
    write_object(&root.join(name.replace('\\', "/")))?;
  }

  fs::write(root.join("notes.txt"), "not an object")?;

  let library: LevelObjectLibrary = LevelObjectLibrary::list(std::slice::from_ref(&root))?;

  assert_eq!(references(&library), ["a\\b\\c", "a\\z", "b", "detail\\grass"]);
  assert!(library.get_refusals().is_empty());
  assert!(library.get_entries()[3].is_under("DETAIL"));
  assert!(!library.get_entries()[2].is_under("detail"));

  Ok(())
}

/// Which file failed and why reaches the scene's refusals, once however many steps read it.
#[test]
fn records_each_object_that_does_not_read_once_with_why() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "unread"))?;

  write_object(&root.join("good.object"))?;
  fs::write(root.join("broken.object"), b"not an object")?;

  let mut library: LevelObjectLibrary = LevelObjectLibrary::list(std::slice::from_ref(&root))?;
  let mut read: Vec<String> = Vec::new();

  for _ in 0..2 {
    library.read_each::<XRayByteOrder>(
      |_| true,
      |entry, _: EditableObjectFile| {
        read.push(entry.reference.clone());

        Ok(())
      },
    )?;
  }

  let refusals = library.get_refusals();

  assert_eq!(read, ["good", "good"]);
  assert_eq!(refusals.len(), 1, "{refusals:?}");
  assert_eq!(refusals[0].object, "broken.object");
  assert!(refusals[0].file.ends_with("broken.object"), "{refusals:?}");
  assert!(!refusals[0].reason.is_empty());

  Ok(())
}

/// A junction or link back to a folder above it, as a hand-made mod layout can hold, would walk forever.
#[test]
fn records_a_link_back_to_a_folder_above_it_rather_than_following_it() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "loop"))?;

  write_object(&root.join("props").join("crate.object"))?;

  if !HostFilesystem::link_directory(&root, &root.join("props").join("again")) {
    return Ok(());
  }

  let library: LevelObjectLibrary = LevelObjectLibrary::list(std::slice::from_ref(&root))?;
  let refusals = library.get_refusals();

  assert_eq!(references(&library), ["props\\crate"]);
  assert_eq!(refusals.len(), 1, "{refusals:?}");
  assert!(refusals[0].reason.contains("links back to"), "{refusals:?}");

  Ok(())
}

/// A junction left behind by a mod removed since is a file not used, not a library that cannot be read.
#[test]
fn records_a_link_to_nothing_rather_than_failing() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "dangling"))?;
  let gone: PathBuf = root.join("gone");

  write_object(&root.join("props").join("crate.object"))?;
  fs::create_dir_all(&gone)?;

  if !HostFilesystem::link_directory(&gone, &root.join("props").join("removed_mod")) {
    return Ok(());
  }

  fs::remove_dir(&gone)?;

  let library: LevelObjectLibrary = LevelObjectLibrary::list(std::slice::from_ref(&root))?;
  let refusals = library.get_refusals();

  assert_eq!(references(&library), ["props\\crate"]);
  assert_eq!(refusals.len(), 1, "{refusals:?}");
  assert!(refusals[0].reason.contains("links to nothing"), "{refusals:?}");

  Ok(())
}

#[test]
fn refuses_a_root_that_is_not_a_directory() {
  assert!(LevelObjectLibrary::list(&[PathBuf::from("no/such/library")]).is_err());
}
