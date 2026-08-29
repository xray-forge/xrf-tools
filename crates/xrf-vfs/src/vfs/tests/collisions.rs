//! Two files in one mount claiming one engine identity, and prefixes a mount omits.
//!
//! Both are properties of a single source: a collision has no priority order to appeal to inside one mount, and ignoring is
//! per source so an override tree can skip what the installation beneath it still serves.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::vfs::tests::fake_source::FakeArchiveSource;
use crate::{
  XrayCollisionSite, XrayLogicalPath, XrayLookupScope, XrayMountPlan, XrayPathCollision, XraySourceKind, XrayVfs,
};

/// Writes a tree whose file names differ only by case, which normalize to one logical path.
fn tree(name: &str, files: &[&str]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_vfs_collisions/{name}"));

  let _ = fs::remove_dir_all(&root);

  for file in files {
    let path: PathBuf = root.join(file);

    fs::create_dir_all(path.parent().expect("file sits in a directory")).expect("tree is creatable");
    fs::write(&path, b"payload").expect("file is writable");
  }

  root
}

#[test]
fn reports_collisions_from_every_mount_in_scope() {
  // A real directory collision needs two paths that differ only by case, which a case-insensitive filesystem cannot hold —
  // so it is a *cross-platform* authoring lint: authored on Windows, surfaced when the tree is read on Linux. That makes a
  // source double the only way to exercise the reporting path here, and the aggregation is what this crate owns.
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount("", Box::new(FakeArchiveSource::new("clean", &["configs/system.ltx"])))
    .expect("clean mounts");
  vfs
    .mount(
      "",
      Box::new(
        FakeArchiveSource::new("clashing", &["textures/wpn/wpn_ak74.dds"]).with_collision(XrayPathCollision {
          kept: XrayCollisionSite::Loose(PathBuf::from("C:\\tree\\textures\\wpn\\wpn_ak74.dds")),
          logical_path: XrayLogicalPath::new("textures\\wpn\\wpn_ak74.dds").expect("valid logical path"),
          unreachable: XrayCollisionSite::Loose(PathBuf::from("C:\\tree\\textures\\Wpn\\wpn_ak74.dds")),
        }),
      ),
    )
    .expect("clashing mounts");

  let collisions: Vec<XrayPathCollision> = vfs.list_collisions();

  assert_eq!(collisions.len(), 1, "reported once, from the mount that holds it");
  assert_eq!(collisions[0].logical_path.as_str(), "textures\\wpn\\wpn_ak74.dds");
  assert!(
    vfs.find("textures\\wpn\\wpn_ak74.dds").expect("lookup").is_some(),
    "resolution is unaffected: one of the two still answers"
  );
}

#[test]
fn a_clean_mount_reports_nothing() {
  let root: PathBuf = tree("clean", &["textures/wpn/wpn_ak74.dds", "configs/system.ltx"]);
  let vfs: XrayVfs = XrayVfs::from_plan(&XrayMountPlan::root(&root).expect("plan")).expect("mounts");

  assert!(vfs.list_collisions().is_empty());

  let _ = fs::remove_dir_all(root);
}

#[test]
fn an_ignored_prefix_is_absent_from_the_mount() {
  let root: PathBuf = tree(
    "ignored",
    &[
      "textures/wpn/wpn_ak74.dds",
      "textures/wip/draft.dds",
      "configs/system.ltx",
    ],
  );

  let plan: XrayMountPlan = XrayMountPlan::root(&root)
    .expect("plan")
    .ignoring(&[String::from("textures\\wip")])
    .expect("prefix is valid");
  let vfs: XrayVfs = XrayVfs::from_plan(&plan).expect("mounts");
  let scope: XrayLookupScope = XrayLookupScope::all();

  assert!(
    vfs
      .scoped(&scope)
      .find("textures\\wip\\draft.dds")
      .expect("lookup")
      .is_none()
  );
  assert!(
    vfs
      .scoped(&scope)
      .find("textures\\wpn\\wpn_ak74.dds")
      .expect("lookup")
      .is_some(),
    "only the named prefix is omitted"
  );
  assert_eq!(vfs.scoped(&scope).list_entries().len(), 2);

  let _ = fs::remove_dir_all(root);
}

#[test]
fn ignoring_matches_on_component_boundaries() {
  // `textures\wip` must not omit `textures\wipers`, or an ignore list would silently hide neighbours.
  let root: PathBuf = tree(
    "ignored_boundary",
    &["textures/wip/draft.dds", "textures/wipers/blade.dds"],
  );

  let plan: XrayMountPlan = XrayMountPlan::root(&root)
    .expect("plan")
    .ignoring(&[String::from("textures\\wip")])
    .expect("prefix is valid");
  let vfs: XrayVfs = XrayVfs::from_plan(&plan).expect("mounts");

  assert_eq!(
    vfs
      .list_entries()
      .iter()
      .map(|entry| entry.get_logical_path().to_string())
      .collect::<Vec<_>>(),
    vec![String::from("textures\\wipers\\blade.dds")]
  );

  let _ = fs::remove_dir_all(root);
}

#[test]
fn rejects_an_ignored_prefix_that_is_not_a_logical_path() {
  let root: PathBuf = tree("ignored_invalid", &["configs/system.ltx"]);

  assert!(
    XrayMountPlan::root(&root)
      .expect("plan")
      .ignoring(&[String::from("configs\\..\\textures")])
      .is_err()
  );

  let _ = fs::remove_dir_all(root);
}

#[test]
fn a_source_that_cannot_be_opened_is_recorded_rather_than_silently_dropped() {
  // Tolerance is only honest if it is visible: a check enumerating a mount that never opened would report its assets
  // as missing content rather than as an unread source.
  let root: PathBuf = tree("skipped", &["configs/system.ltx"]);
  let absent: PathBuf = root.join("nonexistent-volumes");

  let plan: XrayMountPlan = XrayMountPlan::root(&root)
    .expect("plan")
    .with_kind(&absent, "", "$arch_dir$", XraySourceKind::Archive)
    .expect("archive planned");

  let vfs: XrayVfs = XrayVfs::from_plan(&plan).expect("the readable mount still opens");

  assert_eq!(vfs.get_mounts().len(), 1, "only the directory mounted");
  assert_eq!(vfs.get_skipped_mounts().len(), 1);
  assert_eq!(vfs.get_skipped_mounts()[0].origin, "$arch_dir$");
  assert_eq!(vfs.get_skipped_mounts()[0].path, absent);
  assert!(
    !vfs.get_skipped_mounts()[0].reason.is_empty(),
    "the reason is what a report shows a person"
  );

  // The rest of the installation still resolves, which is why skipping rather than failing is the right default.
  assert!(vfs.find("configs\\system.ltx").expect("lookup").is_some());

  let _ = fs::remove_dir_all(root);
}

#[test]
fn enumeration_is_ordered_by_logical_path() {
  // Archive sources key their name tables by hash, so without this the order differs between runs and every consumer
  // that prints or diffs a listing has to remember to sort.
  let mut vfs: XrayVfs = XrayVfs::new();

  vfs
    .mount(
      "",
      Box::new(FakeArchiveSource::new(
        "unordered",
        &[
          "textures\\wpn\\wpn_ak74.dds",
          "configs\\system.ltx",
          "meshes\\actors\\stalker.ogf",
        ],
      )),
    )
    .expect("mounts");

  let ordered: Vec<String> = vfs
    .list_entries()
    .into_iter()
    .map(|entry| entry.get_logical_path().to_string())
    .collect();

  assert_eq!(
    ordered,
    vec![
      "configs\\system.ltx",
      "meshes\\actors\\stalker.ogf",
      "textures\\wpn\\wpn_ak74.dds",
    ]
  );
}
