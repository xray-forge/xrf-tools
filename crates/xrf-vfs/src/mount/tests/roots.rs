#[cfg(any(unix, windows))]
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{XrayMountMode, XrayMountPlan, XrayProbePlan, XrayRoot, XrayRoots, XrayVfs};

/// Builds a loose root holding one asset, since a plan is decided by what is actually there.
fn root(name: &str, logical: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_roots/{name}"));

  let _ = fs::remove_dir_all(&root);

  let path: PathBuf = root.join(logical.replace('\\', "/"));

  fs::create_dir_all(path.parent().expect("entry parent")).expect("root directory");
  fs::write(&path, b"payload").expect("root file");

  root
}

#[cfg(unix)]
fn non_unicode_component() -> OsString {
  use std::os::unix::ffi::OsStringExt;

  OsString::from_vec(b"root-\xff".to_vec())
}

#[cfg(windows)]
fn non_unicode_component() -> OsString {
  use std::os::windows::ffi::OsStringExt;

  let mut name: Vec<u16> = "root-".encode_utf16().collect();
  name.push(0xd800);

  OsString::from_wide(&name)
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
  let spec: XrayRoots = XrayRoots::one(first.clone(), XrayMountMode::Directory);

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

  let spec: XrayRoots = XrayRoots::new([
    XrayRoot::new(front.clone(), XrayMountMode::Directory),
    XrayRoot::new(back.clone(), XrayMountMode::Directory),
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
  let named: PathBuf = only.clone();

  let spec: XrayRoots = XrayRoots::new([
    XrayRoot::new(named.clone(), XrayMountMode::Directory),
    XrayRoot::new(named.clone(), XrayMountMode::Directory),
  ]);

  assert_eq!(
    paths(&spec.to_mount_plan().expect("plan")),
    vec![named.display().to_string()]
  );
}

#[test]
fn defaults_a_root_to_auto() {
  let spec: XrayRoot = XrayRoot::default();

  assert_eq!(spec.mode, XrayMountMode::Auto);
}

#[test]
fn plans_nothing_for_an_empty_spec() {
  let spec: XrayRoots = XrayRoots::default();

  assert!(spec.is_empty());
  assert!(spec.to_mount_plan().expect("plan").is_empty());
  assert!(spec.to_probe_plan().expect("plan").is_empty());
}

#[test]
fn centres_on_an_asset_only_when_none_was_named() {
  let spec: XrayRoots = XrayRoots::default();
  let centred: XrayRoots = spec.centred_on(Some(std::path::Path::new("C:/game/model.ogf")));

  assert_eq!(
    centred.asset.as_deref(),
    Some(std::path::Path::new("C:/game/model.ogf"))
  );

  // A caller that named its own subject keeps it.
  let kept: XrayRoots = centred.centred_on(Some(std::path::Path::new("C:/other/thing.ogf")));

  assert_eq!(kept.asset.as_deref(), Some(std::path::Path::new("C:/game/model.ogf")));
}

#[test]
fn plans_probe_steps_for_an_asset_and_its_roots() {
  let first: PathBuf = root("probe", r"configs\system.ltx");
  let spec: XrayRoots = XrayRoots {
    asset: Some(first.join("configs/system.ltx")),
    roots: vec![XrayRoot::new(first, XrayMountMode::Directory)],
  };

  let plan: XrayProbePlan = spec.to_probe_plan().expect("plan");

  assert!(!plan.is_empty());
}

#[test]
fn opens_what_it_planned() {
  let first: PathBuf = root("open", r"configs\system.ltx");
  let spec: XrayRoots = XrayRoots::one(first, XrayMountMode::Directory);

  let vfs: XrayVfs = spec.open().expect("roots opens");

  assert!(vfs.read_bytes(r"configs\system.ltx").is_ok());
}

#[cfg(any(unix, windows))]
#[test]
fn opens_a_non_unicode_root_without_changing_its_address() {
  let base: PathBuf = build_absolute_generated_test_resource_path("xray_roots/non-unicode-address");
  let root: PathBuf = base.join(non_unicode_component());
  let asset: PathBuf = root.join("configs/system.ltx");

  fs::create_dir_all(asset.parent().expect("entry parent")).expect("non-Unicode root directory");
  fs::write(&asset, b"payload").expect("root file");

  let spec: XrayRoots = XrayRoots::one(root, XrayMountMode::Directory);
  let vfs: XrayVfs = spec.open().expect("roots open");

  assert_eq!(
    vfs
      .read_bytes(r"configs\system.ltx")
      .expect("the declared root remains addressable"),
    b"payload"
  );
}

#[cfg(any(unix, windows))]
#[test]
fn centres_on_a_non_unicode_asset_without_changing_its_address() {
  let asset: PathBuf = PathBuf::from(non_unicode_component()).join("meshes/model.ogf");
  let centred: XrayRoots = XrayRoots::default().centred_on(Some(&asset));

  assert_eq!(centred.asset.as_deref(), Some(asset.as_path()));
}

#[cfg(any(unix, windows))]
#[test]
fn refuses_to_serialize_a_non_unicode_root_as_text() {
  let spec: XrayRoots = XrayRoots::one(PathBuf::from(non_unicode_component()), XrayMountMode::Directory);
  let error: serde_json::Error = serde_json::to_string(&spec).expect_err("non-Unicode root is not a text address");

  assert!(error.to_string().contains("path contains invalid UTF-8 characters"));
}

#[test]
fn survives_a_round_trip_through_serde() {
  // The spec is what a command surface and a stored setting both name, so its wire shape matters.
  let spec: XrayRoots = XrayRoots::new([
    XrayRoot::new(PathBuf::from("C:/gamedata"), XrayMountMode::Directory),
    XrayRoot::new(PathBuf::from("C:/game"), XrayMountMode::ContainingInstallation),
  ]);

  let json: String = serde_json::to_string(&spec).expect("serializes");

  assert!(json.contains("containingInstallation"));
  assert!(json.contains(r#""path":"C:/gamedata""#));
  assert_eq!(serde_json::from_str::<XrayRoots>(&json).expect("deserializes"), spec);
}

#[test]
fn takes_a_root_with_no_mode_as_auto() {
  let spec: XrayRoots =
    serde_json::from_str(r#"{"asset":null,"roots":[{"path":"C:/gamedata"}]}"#).expect("deserializes");

  assert_eq!(spec.roots[0].mode, XrayMountMode::Auto);
}
