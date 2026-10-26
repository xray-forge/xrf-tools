//! Reads a config's include statements through a search, without parsing its sections.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayLookupScope, XraySearch, XrayVfs};

use crate::source::LtxVfsSource;

fn mount_search(name: &str, files: &[(&str, &str)]) -> XraySearch {
  let root: PathBuf = GeneratedResources::create_dir(&format!("ltx_vfs_includes/{name}")).expect("config root");

  for (path, contents) in files {
    let path: PathBuf = root.join(path.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("config directory");
    fs::write(&path, contents).expect("config file");
  }

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &root).expect("root mounts");

  XraySearch::new(vfs).with_step("root", XrayLookupScope::all())
}

#[test]
fn reads_include_statements_and_ignores_sections() {
  let search: XraySearch = mount_search(
    "statements",
    &[(
      "configs\\system.ltx",
      "#include \"first.ltx\"\n;commented\n#include \"second.ltx\"\n[ignored]\nvalue = 1\n",
    )],
  );

  let includes: Vec<String> = LtxVfsSource::new(search.clone())
    .read_included("configs\\system.ltx")
    .expect("includes read");

  assert_eq!(includes, vec!["first.ltx", "second.ltx"]);
}

#[test]
fn answers_an_empty_list_for_a_config_including_nothing() {
  let search: XraySearch = mount_search("none", &[("configs\\system.ltx", "[section]\nvalue = 1\n")]);

  assert_eq!(
    LtxVfsSource::new(search.clone())
      .read_included("configs\\system.ltx")
      .expect("includes read"),
    Vec::<String>::new()
  );
}

#[test]
fn a_config_outside_the_search_is_an_error() {
  let textures: XraySearch = mount_search("out_of_scope", &[("configs\\system.ltx", "[a]\n")])
    .with_prefix("textures")
    .expect("prefix is valid");

  assert!(
    LtxVfsSource::new(textures)
      .read_included("configs\\system.ltx")
      .is_err()
  );
}

#[test]
fn reads_include_statements_declared_after_sections() {
  // What Anomaly's `npc_loadouts.ltx` does: sections first, then the includes naming the files whose parents it defines.
  // Missing them makes every file they name look like nothing includes it.
  let search: XraySearch = mount_search(
    "trailing",
    &[(
      "configs\\system.ltx",
      "#include \"first.ltx\"\n[section]\nvalue = 1\n\n#include \"second.ltx\"\n",
    )],
  );

  assert_eq!(
    LtxVfsSource::new(search.clone())
      .read_included("configs\\system.ltx")
      .expect("includes read"),
    vec!["first.ltx", "second.ltx"]
  );
}
