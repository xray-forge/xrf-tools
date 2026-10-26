//! The vocabulary every archives test is written in: the trees it writes, the searches and worlds it mounts over them,
//! and the volume sets it packs out of them.

use std::fs;
use std::path::PathBuf;

use xrf_pack::{ArchivePackConfig, ArchivePackDirectory, ArchivePacker};
use xrf_test_utils::GeneratedResources;
use xrf_vfs::{XrayRoots, XraySearch, XraySearchStep, XrayVfs};

use crate::browse::ArchiveWorld;

/// A tree of its own holding `files`, emptied first so a re-run reads what this run wrote.
pub fn tree(case: &str, files: &[(&str, &[u8])]) -> PathBuf {
  let root: PathBuf = GeneratedResources::path(&format!("archives/{case}"));

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&root).expect("test tree is creatable");

  for (path, contents) in files {
    let path: PathBuf = root.join(path);

    fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("test tree is creatable");
    fs::write(&path, contents).expect("test file is writable");
  }

  root
}

/// A search over `roots`, mounted as an open mounts them.
pub fn search(roots: &XrayRoots) -> XraySearch {
  let mut vfs: XrayVfs = XrayVfs::new();
  let steps: Vec<XraySearchStep> = roots
    .to_search_plan()
    .expect("the roots plan")
    .mount_into(&mut vfs)
    .expect("the roots mount");

  XraySearch::new(vfs).with_steps(steps)
}

/// The world a search over `roots` lists, read through the same snapshot.
pub fn world(roots: &XrayRoots) -> ArchiveWorld {
  let search: XraySearch = search(roots);

  ArchiveWorld::list(&search, search.clone())
}

/// `files` packed whole into a volume set of their own, as the directory holding its volumes.
pub fn volumes(case: &str, files: &[(&str, &[u8])]) -> PathBuf {
  let source: PathBuf = tree(&format!("{case}/gamedata"), files);
  let destination: PathBuf = GeneratedResources::path(&format!("archives/{case}/db"));
  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, "gamedata");

  let _ = fs::remove_dir_all(&destination);

  config.include_directories = vec![ArchivePackDirectory {
    is_recursive: true,
    path: String::new(),
  }];
  config.is_with_skip_list = false;

  ArchivePacker::pack(&config).expect("the tree packs");

  destination
}
