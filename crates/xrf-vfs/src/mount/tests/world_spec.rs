use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{XrayMountMode, XrayMountPlan, XrayProbePlan, XrayVfs, XrayWorldRoot, XrayWorldSpec};

/// Builds a loose root holding one asset, since a plan is decided by what is actually there.
fn root(name: &str, logical: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_world_spec/{name}"));

  let _ = fs::remove_dir_all(&root);

  let path: PathBuf = root.join(logical.replace('\\', "/"));

  fs::create_dir_all(path.parent().expect("entry parent")).expect("root directory");
  fs::write(&path, b"payload").expect("root file");

  root
}

fn paths(plan: &XrayMountPlan) -> Vec<String> {
  plan
    .get_mounts()
    .iter()
    .map(|mount| mount.path.display().to_string())
    .collect()
}

#[test]
fn plans_one_root() {
  let first: PathBuf = root("single", r"configs\system.ltx");
  let spec: XrayWorldSpec = XrayWorldSpec::root(first.display().to_string(), XrayMountMode::Directory);

  assert_eq!(
    paths(&spec.to_mount_plan().expect("plan")),
    vec![first.display().to_string()]
  );
}

#[test]
fn keeps_roots_in_declaration_order() {
  // Declaration order is search order, which is what layering a tree in front of an installation means.
  let front: PathBuf = root("order-front", r"configs\system.ltx");
  let back: PathBuf = root("order-back", r"configs\system.ltx");

  let spec: XrayWorldSpec = XrayWorldSpec::roots([
    XrayWorldRoot::new(front.display().to_string(), XrayMountMode::Directory),
    XrayWorldRoot::new(back.display().to_string(), XrayMountMode::Directory),
  ]);

  assert_eq!(
    paths(&spec.to_mount_plan().expect("plan")),
    vec![front.display().to_string(), back.display().to_string()]
  );
}

#[test]
fn drops_a_root_named_twice() {
  // `behind` dedupes by path, so a fallback that repeats an earlier root is not mounted twice.
  let only: PathBuf = root("duplicate", r"configs\system.ltx");
  let named: String = only.display().to_string();

  let spec: XrayWorldSpec = XrayWorldSpec::roots([
    XrayWorldRoot::new(named.clone(), XrayMountMode::Directory),
    XrayWorldRoot::new(named.clone(), XrayMountMode::Directory),
  ]);

  assert_eq!(paths(&spec.to_mount_plan().expect("plan")), vec![named]);
}

#[test]
fn defaults_a_root_to_auto() {
  let spec: XrayWorldRoot = XrayWorldRoot::default();

  assert_eq!(spec.mode, XrayMountMode::Auto);
}

#[test]
fn plans_nothing_for_an_empty_spec() {
  let spec: XrayWorldSpec = XrayWorldSpec::default();

  assert!(spec.is_empty());
  assert!(spec.to_mount_plan().expect("plan").is_empty());
  assert!(spec.to_probe_plan().expect("plan").is_empty());
}

#[test]
fn centres_on_an_asset_only_when_none_was_named() {
  let spec: XrayWorldSpec = XrayWorldSpec::default();
  let centred: XrayWorldSpec = spec.centred_on(Some(std::path::Path::new("C:/game/model.ogf")));

  assert_eq!(centred.asset.as_deref(), Some("C:/game/model.ogf"));

  // A caller that named its own subject keeps it.
  let kept: XrayWorldSpec = centred.centred_on(Some(std::path::Path::new("C:/other/thing.ogf")));

  assert_eq!(kept.asset.as_deref(), Some("C:/game/model.ogf"));
}

#[test]
fn plans_probe_steps_for_an_asset_and_its_roots() {
  let first: PathBuf = root("probe", r"configs\system.ltx");
  let spec: XrayWorldSpec = XrayWorldSpec {
    asset: Some(first.join("configs/system.ltx").display().to_string()),
    roots: vec![XrayWorldRoot::new(
      first.display().to_string(),
      XrayMountMode::Directory,
    )],
  };

  let plan: XrayProbePlan = spec.to_probe_plan().expect("plan");

  assert!(!plan.is_empty());
}

#[test]
fn opens_a_world_it_planned() {
  let first: PathBuf = root("open", r"configs\system.ltx");
  let spec: XrayWorldSpec = XrayWorldSpec::root(first.display().to_string(), XrayMountMode::Directory);

  let vfs: XrayVfs = spec.open().expect("world opens");

  assert!(vfs.read(r"configs\system.ltx").is_ok());
}

#[test]
fn survives_a_round_trip_through_serde() {
  // The spec is what a command surface and a stored setting both name, so its wire shape matters.
  let spec: XrayWorldSpec = XrayWorldSpec::roots([
    XrayWorldRoot::new("C:/gamedata", XrayMountMode::Directory),
    XrayWorldRoot::new("C:/game", XrayMountMode::ContainingInstallation),
  ]);

  let json: String = serde_json::to_string(&spec).expect("serializes");

  assert!(json.contains("containingInstallation"));
  assert_eq!(
    serde_json::from_str::<XrayWorldSpec>(&json).expect("deserializes"),
    spec
  );
}

#[test]
fn takes_a_root_with_no_mode_as_auto() {
  let spec: XrayWorldSpec =
    serde_json::from_str(r#"{"asset":null,"roots":[{"path":"C:/gamedata"}]}"#).expect("deserializes");

  assert_eq!(spec.roots[0].mode, XrayMountMode::Auto);
}
