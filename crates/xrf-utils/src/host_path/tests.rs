//! A destination under either link policy, against trees that already hold links.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfError;
use xrf_test_utils::links::{link_directory, link_file};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{HostDestination, HostLinkPolicy, HostRelativePath};

fn create_directory(name: &str) -> PathBuf {
  let directory: PathBuf = build_absolute_generated_test_resource_path(&format!("host-path/{name}"));

  let _ = fs::remove_dir_all(&directory);

  fs::create_dir_all(&directory).expect("temporary directory");

  directory
}

/// A directory outside the root, holding a file no write is allowed to touch unless the layout sends it there.
fn create_sentinel(directory: &Path) -> PathBuf {
  let sentinel: PathBuf = directory.join("sentinel");

  fs::create_dir_all(&sentinel).expect("sentinel directory");
  fs::write(sentinel.join("keep.txt"), b"original").expect("sentinel file");

  sentinel
}

fn relative(name: &str) -> HostRelativePath {
  HostRelativePath::new(name, "Entry").expect("a safe relative path")
}

fn write(destination: &HostDestination, name: &str, contents: &[u8]) -> Result<(), XrfError> {
  use std::io::Write;

  destination.create_file(&relative(name))?.write_all(contents)?;

  Ok(())
}

#[test]
fn follow_writes_through_a_directory_link_the_user_placed() {
  let directory: PathBuf = create_directory("follow-directory-link");
  let sentinel: PathBuf = create_sentinel(&directory);
  let root: PathBuf = directory.join("out");

  fs::create_dir_all(&root).expect("destination");

  // A junctioned texture tree or a symlink deployment: the layout is the user's, so the write lands where it points.
  if !link_directory(&sentinel, &root.join("textures")) {
    return;
  }

  write(
    &HostDestination::new(&root, HostLinkPolicy::Follow),
    "textures\\ui\\icon.dds",
    b"dds",
  )
  .expect("write");

  assert_eq!(
    fs::read(sentinel.join("ui").join("icon.dds")).expect("written file"),
    b"dds"
  );
}

#[test]
fn follow_overwrites_a_file_through_its_link() {
  let directory: PathBuf = create_directory("follow-file-link");
  let sentinel: PathBuf = create_sentinel(&directory);
  let root: PathBuf = directory.join("out");

  fs::create_dir_all(&root).expect("destination");

  if !link_file(&sentinel.join("keep.txt"), &root.join("keep.txt")) {
    return;
  }

  write(
    &HostDestination::new(&root, HostLinkPolicy::Follow),
    "keep.txt",
    b"replaced",
  )
  .expect("write");

  assert_eq!(
    fs::read_to_string(sentinel.join("keep.txt")).expect("sentinel file"),
    "replaced"
  );
}

#[test]
fn refuse_stops_at_a_directory_link() {
  let directory: PathBuf = create_directory("refuse-directory-link");
  let sentinel: PathBuf = create_sentinel(&directory);
  let root: PathBuf = directory.join("out");

  fs::create_dir_all(&root).expect("destination");

  if !link_directory(&sentinel, &root.join("configs")) {
    return;
  }

  let destination: HostDestination = HostDestination::new(&root, HostLinkPolicy::Refuse);

  assert!(matches!(
    write(&destination, "configs\\system.ltx", b"[section]"),
    Err(XrfError::Invalid { .. })
  ));
  assert!(matches!(
    destination.prepare_file(&relative("configs\\system.ltx")),
    Err(XrfError::Invalid { .. })
  ));
  assert!(!sentinel.join("system.ltx").exists(), "a write crossed a refused link");
}

#[test]
fn refuse_stops_at_a_file_link() {
  let directory: PathBuf = create_directory("refuse-file-link");
  let sentinel: PathBuf = create_sentinel(&directory);
  let root: PathBuf = directory.join("out");

  fs::create_dir_all(&root).expect("destination");

  if !link_file(&sentinel.join("keep.txt"), &root.join("keep.txt")) {
    return;
  }

  let destination: HostDestination = HostDestination::new(&root, HostLinkPolicy::Refuse);

  assert!(write(&destination, "keep.txt", b"replaced").is_err());
  assert!(destination.prepare_file(&relative("keep.txt")).is_err());
  assert_eq!(
    fs::read_to_string(sentinel.join("keep.txt")).expect("sentinel file"),
    "original"
  );
}

/// The walk checks one component at a time, so an ordinary tree that is already there must keep accepting writes.
#[test]
fn writes_into_an_existing_tree_under_either_policy() {
  for policy in [HostLinkPolicy::Follow, HostLinkPolicy::Refuse] {
    let directory: PathBuf = create_directory(&format!("existing-tree-{policy:?}"));
    let root: PathBuf = directory.join("out");

    fs::create_dir_all(root.join("configs")).expect("destination");
    fs::write(root.join("configs").join("system.ltx"), b"stale").expect("previous run");

    let destination: HostDestination = HostDestination::new(&root, policy);

    write(&destination, "configs/system.ltx", b"[section]").expect("write");

    assert_eq!(
      fs::read_to_string(root.join("configs").join("system.ltx")).expect("written file"),
      "[section]"
    );
    assert_eq!(
      destination.prepare_file(&relative("ui\\icon.dds")).expect("prepared"),
      root.join("ui").join("icon.dds")
    );
    assert!(
      root.join("ui").is_dir(),
      "preparing a file creates the directories above it"
    );
  }
}

#[test]
fn refuses_a_component_that_is_a_file() {
  let directory: PathBuf = create_directory("component-is-file");
  let root: PathBuf = directory.join("out");

  fs::create_dir_all(&root).expect("destination");
  fs::write(root.join("configs"), b"not a directory").expect("blocking file");

  for policy in [HostLinkPolicy::Follow, HostLinkPolicy::Refuse] {
    assert!(matches!(
      write(
        &HostDestination::new(&root, policy),
        "configs\\system.ltx",
        b"[section]"
      ),
      Err(XrfError::Invalid { .. })
    ));
  }
}

#[test]
fn refuses_a_path_that_names_no_file() {
  let directory: PathBuf = create_directory("no-file-name");
  let destination: HostDestination = HostDestination::new(&directory, HostLinkPolicy::Follow);

  assert!(destination.create_file(&HostRelativePath::default()).is_err());
  assert!(destination.prepare_file(&HostRelativePath::default()).is_err());
}

/// `create_file_in` skips the walk down because its caller already verified the parent, so it must not accept any
/// parent or name a caller offers.
#[test]
fn create_file_in_refuses_a_parent_outside_the_root_or_a_path_for_a_name() {
  let directory: PathBuf = create_directory("parent-outside");
  let root: PathBuf = directory.join("out");
  let outside: PathBuf = directory.join("elsewhere");

  fs::create_dir_all(&outside).expect("a directory beside the root");

  let destination: HostDestination = HostDestination::new(&root, HostLinkPolicy::Follow);

  destination.create_root().expect("the root is created once");

  assert!(matches!(
    destination.create_file_in(&outside, OsStr::new("system.ltx")),
    Err(XrfError::Invalid { .. })
  ));
  assert!(
    !outside.join("system.ltx").exists(),
    "a refused parent was written into"
  );
  assert!(matches!(
    destination.create_file_in(&root.join("..").join("elsewhere"), OsStr::new("system.ltx")),
    Err(XrfError::Invalid { .. })
  ));
  assert!(
    !outside.join("system.ltx").exists(),
    "a parent climbing out of the root was written into"
  );
  assert!(matches!(
    destination.create_file_in(&root, OsStr::new("system:stream")),
    Err(XrfError::Invalid { .. })
  ));

  let escape: PathBuf = Path::new("..").join("elsewhere").join("system.ltx");

  assert!(matches!(
    destination.create_file_in(&root, escape.as_os_str()),
    Err(XrfError::Invalid { .. })
  ));
  assert!(!outside.join("system.ltx").exists(), "a refused name was written");

  destination
    .create_file_in(&root, OsStr::new("system.ltx"))
    .expect("a parent below the root is accepted");

  assert!(root.join("system.ltx").is_file());
}

/// Resolving is for reading a target or a dry run, so it must leave the destination exactly as it found it.
#[test]
fn resolve_creates_nothing_under_either_policy() {
  for policy in [HostLinkPolicy::Follow, HostLinkPolicy::Refuse] {
    let directory: PathBuf = create_directory(&format!("resolve-creates-nothing-{policy:?}"));
    let root: PathBuf = directory.join("out");
    let destination: HostDestination = HostDestination::new(&root, policy);

    assert_eq!(
      destination
        .resolve_file(&relative("text/eng/st_items.json"))
        .expect("resolved"),
      root.join("text").join("eng").join("st_items.json")
    );
    assert!(!root.exists(), "resolving created the root");
    assert!(destination.resolve_file(&HostRelativePath::default()).is_err());
  }
}

#[test]
fn resolve_applies_the_policy_to_links_that_exist() {
  let directory: PathBuf = create_directory("resolve-links");
  let sentinel: PathBuf = create_sentinel(&directory);
  let root: PathBuf = directory.join("out");

  fs::create_dir_all(&root).expect("destination");

  if !link_directory(&sentinel, &root.join("text")) || !link_file(&sentinel.join("keep.txt"), &root.join("keep.txt")) {
    return;
  }

  let following: HostDestination = HostDestination::new(&root, HostLinkPolicy::Follow);
  let refusing: HostDestination = HostDestination::new(&root, HostLinkPolicy::Refuse);

  assert_eq!(
    following
      .resolve_file(&relative("text/eng/st_items.json"))
      .expect("followed"),
    root.join("text").join("eng").join("st_items.json")
  );
  assert!(following.resolve_file(&relative("keep.txt")).is_ok());
  assert!(matches!(
    refusing.resolve_file(&relative("text/eng/st_items.json")),
    Err(XrfError::Invalid { .. })
  ));
  assert!(matches!(
    refusing.resolve_file(&relative("keep.txt")),
    Err(XrfError::Invalid { .. })
  ));
}
