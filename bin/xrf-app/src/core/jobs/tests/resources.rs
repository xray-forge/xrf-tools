use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use uuid::Uuid;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::core::jobs::{JobKind, JobRegistry, JobResource, JobStart, resolve_lease_path};

fn scratch() -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path("job_resources").join(Uuid::new_v4().to_string());
  fs::create_dir_all(&root).expect("scratch directory");
  root
}

#[test]
fn file_and_tree_claims_overlap_in_both_registration_orders() {
  let root: PathBuf = scratch();
  let claims = [
    JobResource::tree(&root),
    JobResource::file(root.join("textures/test.dds")),
  ];
  for (first, second) in [(&claims[0], &claims[1]), (&claims[1], &claims[0])] {
    let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
    let (_job, registration) = registry
      .register(JobStart::new(Uuid::new_v4(), JobKind::ArchivesPack).with_resources(vec![first.clone()]))
      .expect("first claim");
    assert!(
      registry
        .register(JobStart::new(Uuid::new_v4(), JobKind::ArchivesUnpack).with_resources(vec![second.clone()]))
        .is_err()
    );
    drop(registration);
    registry
      .register(JobStart::new(Uuid::new_v4(), JobKind::ArchivesUnpack).with_resources(vec![second.clone()]))
      .expect("released claim");
  }
}

#[test]
fn similarly_named_sibling_trees_do_not_overlap() {
  let root: PathBuf = scratch();
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let (_job, _registration) = registry
    .register(
      JobStart::new(Uuid::new_v4(), JobKind::ArchivesPack)
        .with_resources(vec![JobResource::tree(root.join("textures"))]),
    )
    .expect("first tree");
  registry
    .register(
      JobStart::new(Uuid::new_v4(), JobKind::ArchivesUnpack)
        .with_resources(vec![JobResource::tree(root.join("textures-backup"))]),
    )
    .expect("sibling tree");
}

#[test]
fn a_missing_destination_keeps_its_identity_after_creation() {
  let root: PathBuf = scratch();
  let path: PathBuf = root.join("new/sub/texture.dds");
  let before: PathBuf = resolve_lease_path(&path).expect("missing path");
  fs::create_dir_all(path.parent().expect("parent")).expect("new parents");
  fs::write(&path, []).expect("new file");
  assert_eq!(before, resolve_lease_path(&path).expect("existing path"));
  assert_eq!(
    before,
    resolve_lease_path(&root.join("new/unused/../sub/./texture.dds")).expect("lexical alias")
  );
}

#[test]
fn an_invalid_resource_takes_neither_the_group_nor_other_resources() {
  let root: PathBuf = scratch();
  fs::write(root.join("file"), []).expect("file, not directory");
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  assert!(
    registry
      .register(
        JobStart::new(Uuid::new_v4(), JobKind::ArchivesPack)
          .with_exclusion_group("writer")
          .with_resources(vec![
            JobResource::file(root.join("free")),
            JobResource::file(root.join("file/child"))
          ])
      )
      .is_err()
  );
  registry
    .register(
      JobStart::new(Uuid::new_v4(), JobKind::ArchivesUnpack)
        .with_exclusion_group("writer")
        .with_resources(vec![JobResource::file(root.join("free"))]),
    )
    .expect("refusal retained nothing");
}

#[test]
fn a_duplicate_live_identity_cannot_replace_its_claims() {
  let root: PathBuf = scratch();
  let registry: Arc<JobRegistry> = Arc::new(JobRegistry::new());
  let id: Uuid = Uuid::new_v4();
  let (_job, _registration) = registry
    .register(JobStart::new(id, JobKind::ArchivesPack).with_resources(vec![JobResource::tree(&root)]))
    .expect("original job");
  assert!(registry.register(JobStart::new(id, JobKind::ArchivesUnpack)).is_err());
  assert!(
    registry
      .register(
        JobStart::new(Uuid::new_v4(), JobKind::ArchivesUnpack)
          .with_resources(vec![JobResource::file(root.join("file"))])
      )
      .is_err()
  );
}

#[test]
fn path_case_follows_the_host() {
  let root: PathBuf = scratch();
  let upper: PathBuf = resolve_lease_path(&root.join("Texture.dds")).expect("upper");
  let lower: PathBuf = resolve_lease_path(&root.join("texture.dds")).expect("lower");
  #[cfg(windows)]
  assert_eq!(upper, lower);
  #[cfg(not(windows))]
  assert_ne!(upper, lower);
}

#[test]
fn a_filesystem_root_contains_its_canonical_children() {
  let child: PathBuf = scratch();
  let root = child.ancestors().last().expect("filesystem root");
  assert!(
    resolve_lease_path(&child)
      .expect("child")
      .starts_with(resolve_lease_path(root).expect("root"))
  );
}

#[cfg(unix)]
#[test]
fn symlinked_ancestors_resolve_missing_children_and_parent_components() {
  use std::os::unix::fs::symlink;

  let root: PathBuf = scratch();
  fs::create_dir_all(root.join("real/sub")).expect("target");
  symlink(root.join("real/sub"), root.join("alias")).expect("symlink");
  assert_eq!(
    resolve_lease_path(&root.join("alias/new.dds")).expect("alias"),
    resolve_lease_path(&root.join("real/sub/new.dds")).expect("target")
  );
  assert_eq!(
    resolve_lease_path(&root.join("alias/../new.dds")).expect("alias parent"),
    resolve_lease_path(&root.join("real/new.dds")).expect("target parent")
  );
  symlink(root.join("absent"), root.join("dangling")).expect("dangling symlink");
  assert!(resolve_lease_path(&root.join("dangling/new.dds")).is_err());
}
