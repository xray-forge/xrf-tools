use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{XrayAssetType, XrayProbePlan, XrayProbeStep, XrayResolution, XrayVfs};

/// A loose X-Ray root, which is what an implied plan recognizes: a tree holding both `meshes` and `textures`.
fn root(name: &str, files: &[&str]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_probe_plan/{name}"));

  let _ = fs::remove_dir_all(&root);

  for directory in ["meshes", "textures"] {
    fs::create_dir_all(root.join(directory)).expect("root holds the directories an implied plan looks for");
  }

  for file in files {
    let path: PathBuf = root.join(file);

    fs::create_dir_all(path.parent().expect("file sits in a directory")).expect("test tree is creatable");
    fs::write(&path, name.as_bytes()).expect("test file is writable");
  }

  root
}

#[test]
fn searches_an_asset_own_root_before_a_declared_one() {
  let beside: PathBuf = root("beside", &["meshes/wpn/wpn_ak74.ogf", "textures/wpn/wpn_ak74.dds"]);
  let project: PathBuf = root("project", &["textures/wpn/wpn_ak74.dds"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  let steps: Vec<XrayProbeStep> = XrayProbePlan::new()
    .with_asset(beside.join("meshes").join("wpn").join("wpn_ak74.ogf"))
    .expect("asset plans")
    .with_root("project gamedata", &project)
    .expect("root plans")
    .mount_into(&mut vfs)
    .expect("plan mounts");

  let resolution: XrayResolution = vfs
    .probe()
    .with_steps(steps)
    .resolve(XrayAssetType::Dds, "wpn\\wpn_ak74")
    .expect("lookup succeeds");

  assert_eq!(resolution.get_step(), Some(XrayProbePlan::ASSET_STEP));
  assert_eq!(
    resolution.get_asset().and_then(|it| it.get_root()),
    Some(beside.as_path()),
    "a texture beside the model wins over the same name in the project"
  );
}

#[test]
fn falls_through_to_a_declared_root_for_what_the_asset_root_lacks() {
  let beside: PathBuf = root("beside_partial", &["meshes/wpn/wpn_ak74.ogf"]);
  let project: PathBuf = root("project_partial", &["textures/wpn/wpn_ak74.dds"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  let steps: Vec<XrayProbeStep> = XrayProbePlan::new()
    .with_asset(beside.join("meshes").join("wpn").join("wpn_ak74.ogf"))
    .expect("asset plans")
    .with_root("project gamedata", &project)
    .expect("root plans")
    .mount_into(&mut vfs)
    .expect("plan mounts");

  let resolution: XrayResolution = vfs
    .probe()
    .with_steps(steps)
    .resolve(XrayAssetType::Dds, "wpn\\wpn_ak74")
    .expect("lookup succeeds");

  assert_eq!(resolution.get_step(), Some("project gamedata"));
  assert_eq!(
    resolution.get_asset().and_then(|it| it.get_root()),
    Some(project.as_path())
  );
}

#[test]
fn a_root_that_is_not_a_directory_plans_a_step_that_searches_nothing() {
  let mut vfs: XrayVfs = XrayVfs::new();

  let steps: Vec<XrayProbeStep> = XrayProbePlan::new()
    .with_root(
      "unconfigured",
      PathBuf::from("C:").join("does").join("not").join("exist"),
    )
    .expect("an absent root is not an error")
    .mount_into(&mut vfs)
    .expect("plan mounts");

  assert_eq!(steps.len(), 1, "the step stays visible even though it holds nothing");
  assert_eq!(
    vfs
      .probe()
      .with_steps(steps)
      .resolve(XrayAssetType::Dds, "wpn\\wpn_ak74")
      .expect("lookup succeeds"),
    XrayResolution::NoScope
  );
}

#[test]
fn planning_the_same_root_twice_mounts_it_once() {
  let project: PathBuf = root("reused", &["textures/wpn/wpn_ak74.dds"]);

  let mut vfs: XrayVfs = XrayVfs::new();

  // A viewer plans per asset, so the same project root is planned again for every model opened under it.
  for _ in 0..3 {
    XrayProbePlan::new()
      .with_root("project gamedata", &project)
      .expect("root plans")
      .mount_into(&mut vfs)
      .expect("plan mounts");
  }

  assert_eq!(vfs.get_mounts().len(), 1, "mounting is idempotent per planned path");
}

#[test]
fn finds_the_steps_of_a_plan_only_once_it_is_mounted() {
  let beside: PathBuf = root("found_beside", &["meshes/wpn/wpn_ak74.ogf"]);
  let project: PathBuf = root("found_project", &["textures/wpn/wpn_ak74.dds"]);
  let plan: XrayProbePlan = XrayProbePlan::new()
    .with_asset(beside.join("meshes").join("wpn").join("wpn_ak74.ogf"))
    .expect("asset plans")
    .with_root("project gamedata", &project)
    .expect("root plans");

  let mut vfs: XrayVfs = XrayVfs::new();

  assert_eq!(plan.find_mounted(&vfs), None);

  let steps: Vec<XrayProbeStep> = plan.mount_into(&mut vfs).expect("plan mounts");

  assert_eq!(plan.find_mounted(&vfs), Some(steps));
}
