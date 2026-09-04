//! Reads a config's include statements out of a VFS, without parsing its sections.
//!
//! Project assembly needs every config's include list to work out which files nothing includes, and none of their
//! contents, so that pass reads them separately and cheaply.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayLookupScope, XrayVfs};

use crate::ltx::Ltx;

fn mount(name: &str, files: &[(&str, &str)]) -> (XrayVfs, XrayLookupScope) {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("ltx_vfs_includes/{name}"));

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
fn reads_include_statements_and_ignores_sections() {
  let (vfs, scope) = mount(
    "statements",
    &[(
      "configs\\system.ltx",
      "#include \"first.ltx\"\n;commented\n#include \"second.ltx\"\n[ignored]\nvalue = 1\n",
    )],
  );

  let includes: Vec<String> = Ltx::read_included_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("includes read");

  assert_eq!(includes, vec!["first.ltx", "second.ltx"]);
}

#[test]
fn answers_an_empty_list_for_a_config_including_nothing() {
  let (vfs, scope) = mount("none", &[("configs\\system.ltx", "[section]\nvalue = 1\n")]);

  assert_eq!(
    Ltx::read_included_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("includes read"),
    Vec::<String>::new()
  );
}

#[test]
fn a_config_outside_the_scope_is_an_error() {
  let (vfs, scope) = mount("out_of_scope", &[("configs\\system.ltx", "[a]\n")]);
  let textures: XrayLookupScope = scope.with_prefix("textures").expect("prefix is valid");

  assert!(Ltx::read_included_from_vfs(&vfs, &textures, "configs\\system.ltx").is_err());
}

#[test]
fn reads_include_statements_declared_after_sections() {
  // What Anomaly's `npc_loadouts.ltx` does: sections first, then the includes naming the files whose parents it defines.
  // Missing them makes every file they name look like nothing includes it.
  let (vfs, scope) = mount(
    "trailing",
    &[(
      "configs\\system.ltx",
      "#include \"first.ltx\"\n[section]\nvalue = 1\n\n#include \"second.ltx\"\n",
    )],
  );

  assert_eq!(
    Ltx::read_included_from_vfs(&vfs, &scope, "configs\\system.ltx").expect("includes read"),
    vec!["first.ltx", "second.ltx"]
  );
}
