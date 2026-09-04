//! Reads LTX through a mounted VFS, which is how configs come out of an installation.
//!
//! Trees are built on disk and mounted as directories rather than packed into volumes: what these check is include
//! resolution over logical paths, and a directory source exercises that identically to an archive one while staying fast.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayLookupScope, XrayVfs};

use crate::ltx::Ltx;

/// Builds a config tree and mounts it, returning the VFS and an all-mounts scope.
fn mount(name: &str, files: &[(&str, &str)]) -> (XrayVfs, XrayLookupScope) {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("ltx_vfs_read/{name}"));

  let _ = fs::remove_dir_all(&root);

  for (path, contents) in files {
    let path: PathBuf = root.join(path.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("config directory");
    fs::write(&path, contents).expect("config file");
  }

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("root mounts");

  (vfs, XrayLookupScope::all())
}

#[test]
fn reads_a_config_with_no_includes() {
  let (vfs, scope) = mount("plain", &[("configs\\system.ltx", "[section]\nvalue = 1\n")]);

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("config reads");

  assert_eq!(ltx.get_from("section", "value"), Some("1"));
}

#[test]
fn resolves_a_named_include_against_the_logical_directory() {
  let (vfs, scope) = mount(
    "named",
    &[
      (
        "configs\\system.ltx",
        "#include \"sections\\first.ltx\"\n[own]\nvalue = 1\n",
      ),
      ("configs\\sections\\first.ltx", "[first]\nvalue = 2\n"),
    ],
  );

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("config reads");

  assert_eq!(ltx.get_from("first", "value"), Some("2"), "the include was merged");
  assert_eq!(ltx.get_from("own", "value"), Some("1"));
}

#[test]
fn expands_a_wildcard_include_by_enumerating_its_directory() {
  // The case that cannot work through `read_dir` once configs live in volumes.
  let (vfs, scope) = mount(
    "wildcard",
    &[
      ("configs\\system.ltx", "#include \"sections\\w_*.ltx\"\n"),
      ("configs\\sections\\w_first.ltx", "[first]\nvalue = 1\n"),
      ("configs\\sections\\w_second.ltx", "[second]\nvalue = 2\n"),
      ("configs\\sections\\other.ltx", "[other]\nvalue = 3\n"),
    ],
  );

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("config reads");

  assert_eq!(ltx.get_from("first", "value"), Some("1"));
  assert_eq!(ltx.get_from("second", "value"), Some("2"));
  assert_eq!(
    ltx.get_from("other", "value"),
    None,
    "a name the mask does not match stays out"
  );
}

#[test]
fn a_wildcard_include_does_not_reach_into_nested_directories() {
  // A prefix scope answers everything below it, but the statement names one directory. Without a direct-child check a mask
  // would pull in files the engine never loads.
  let (vfs, scope) = mount(
    "nested",
    &[
      ("configs\\system.ltx", "#include \"sections\\w_*.ltx\"\n"),
      ("configs\\sections\\w_first.ltx", "[first]\nvalue = 1\n"),
      ("configs\\sections\\deeper\\w_second.ltx", "[second]\nvalue = 2\n"),
    ],
  );

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("config reads");

  assert_eq!(ltx.get_from("first", "value"), Some("1"));
  assert_eq!(ltx.get_from("second", "value"), None);
}

#[test]
fn resolves_includes_recursively_through_the_same_source() {
  let (vfs, scope) = mount(
    "recursive",
    &[
      ("configs\\system.ltx", "#include \"first.ltx\"\n"),
      ("configs\\first.ltx", "#include \"second.ltx\"\n[first]\nvalue = 1\n"),
      ("configs\\second.ltx", "[second]\nvalue = 2\n"),
    ],
  );

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("config reads");

  assert_eq!(ltx.get_from("first", "value"), Some("1"));
  assert_eq!(
    ltx.get_from("second", "value"),
    Some("2"),
    "a nested include resolves through the vfs too, not the filesystem"
  );
}

#[test]
fn an_include_the_vfs_does_not_hold_is_nothing_to_merge() {
  // The same tolerance the filesystem source shows a config not yet generated from TypeScript.
  let (vfs, scope) = mount(
    "absent",
    &[("configs\\system.ltx", "#include \"generated.ltx\"\n[own]\nvalue = 1\n")],
  );

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("config reads");

  assert_eq!(ltx.get_from("own", "value"), Some("1"));
}

#[test]
fn a_config_outside_the_scope_is_an_error_rather_than_an_empty_result() {
  let (vfs, scope) = mount("out_of_scope", &[("configs\\system.ltx", "[section]\nvalue = 1\n")]);
  let textures: XrayLookupScope = scope.with_prefix("textures").expect("prefix is valid");

  assert!(Ltx::read_from_vfs(&vfs, &textures, "configs\\system.ltx").is_err());
}

#[test]
fn records_the_logical_path_it_was_read_from() {
  // Nested includes resolve against this, so it has to be the logical location rather than a filesystem one.
  let (vfs, scope) = mount("logical", &[("configs\\system.ltx", "[section]\nvalue = 1\n")]);

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("config reads");

  assert_eq!(
    ltx.path.as_deref(),
    Some(PathBuf::from("configs\\system.ltx").as_path())
  );
  assert_eq!(ltx.directory.as_deref(), Some(PathBuf::from("configs").as_path()));
}

#[test]
fn records_that_location_as_the_normalized_identity() {
  // The recorded location is an engine identity, so it is derived by logical rules rather than host ones. Normalization is the
  // half of that a Windows run can see; the separator half only shows up where `\` does not split a path.
  let (vfs, scope) = mount(
    "normalized",
    &[("configs\\system.ltx", "#include \"sections\\first.ltx\"\n")],
  );

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &scope, "Configs\\System.LTX").expect("config reads");

  assert_eq!(
    ltx.path.as_deref(),
    Some(PathBuf::from("configs\\system.ltx").as_path())
  );
  assert_eq!(ltx.directory.as_deref(), Some(PathBuf::from("configs").as_path()));
}

#[test]
fn a_loose_config_overrides_one_lower_in_the_mount_order() {
  // What the whole layer is for: an override in front of a base tree is what the reader sees.
  let overlay: PathBuf = build_absolute_generated_test_resource_path("ltx_vfs_read/override_overlay");
  let base: PathBuf = build_absolute_generated_test_resource_path("ltx_vfs_read/override_base");

  for (root, value) in [(&overlay, "overridden"), (&base, "base")] {
    let _ = fs::remove_dir_all(root);

    fs::create_dir_all(root.join("configs")).expect("config directory");
    fs::write(root.join("configs/system.ltx"), format!("[section]\nvalue = {value}\n")).expect("config file");
  }

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &overlay).expect("overlay mounts");
  vfs.mount_directory("", &base).expect("base mounts");

  let ltx: Ltx = Ltx::read_from_vfs(&vfs, &XrayLookupScope::all(), "configs\\system.ltx").expect("config reads");

  assert_eq!(ltx.get_from("section", "value"), Some("overridden"));
}
