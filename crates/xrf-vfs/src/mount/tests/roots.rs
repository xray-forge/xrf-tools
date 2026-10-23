#[cfg(any(unix, windows))]
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::{XrayMountMode, XrayRoot, XrayRoots, XraySearch, XraySearchPlan};

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

/// The roots a search over `spec` reaches, in search order.
fn searched_roots(spec: &XrayRoots) -> Vec<String> {
  open(spec).list_roots()
}

fn open(spec: &XrayRoots) -> XraySearch {
  spec.to_search_plan().expect("roots plan").open().expect("roots open")
}

#[test]
fn plans_one_root() {
  let first: PathBuf = root("single", r"configs\system.ltx");
  let spec: XrayRoots = XrayRoots::one(first.clone(), XrayMountMode::Directory);

  assert_eq!(searched_roots(&spec), vec![first.display().to_string()]);
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
    searched_roots(&spec),
    vec![front.display().to_string(), back.display().to_string()]
  );
  assert_eq!(
    open(&spec)
      .find(r"configs\system.ltx")
      .expect("lookup")
      .and_then(|asset| asset.get_root().map(PathBuf::from)),
    Some(front),
    "the front root answers what both hold"
  );
}

#[test]
fn mounts_a_root_named_twice_once() {
  // A fallback that repeats an earlier root is a second step over the same mount, not a second mount.
  let named: PathBuf = root("duplicate", r"configs\system.ltx");

  let spec: XrayRoots = XrayRoots::new([
    XrayRoot::new(named.clone(), XrayMountMode::Directory),
    XrayRoot::new(named.clone(), XrayMountMode::Directory),
  ]);
  let search: XraySearch = open(&spec);

  assert_eq!(search.get_steps().len(), 2);
  assert_eq!(search.get_vfs().get_mounts().len(), 1);
  assert_eq!(searched_roots(&spec), vec![named.display().to_string()]);
}

#[test]
fn a_root_that_is_not_there_is_an_empty_step_reported_as_skipped() {
  // A missing root searches nothing rather than failing the open, and says so, so a report can name what was not read.
  let present: PathBuf = root("missing-present", r"configs\system.ltx");
  let absent: PathBuf = present.with_file_name("missing-absent");

  let spec: XrayRoots = XrayRoots::new([
    XrayRoot::new(absent.clone(), XrayMountMode::Installation),
    XrayRoot::new(present.clone(), XrayMountMode::Directory),
  ]);
  let search: XraySearch = open(&spec);

  assert_eq!(search.get_steps().len(), 2);
  assert_eq!(
    search
      .list_skipped_sources()
      .iter()
      .map(|skipped| skipped.path.clone())
      .collect::<Vec<PathBuf>>(),
    vec![absent],
    "the missing root is skipped, whatever mode named it"
  );
  assert!(search.find(r"configs\system.ltx").expect("lookup").is_some());
  assert!(
    !spec
      .to_search_plan()
      .expect("roots plan")
      .has_failed_in(search.get_vfs()),
    "a root that is not there is not a source that failed to open, so a retry has nothing to try"
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
  assert!(spec.to_search_plan().expect("plan").is_empty());
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
fn plans_search_steps_for_an_asset_and_its_roots() {
  let first: PathBuf = root("search", r"configs\system.ltx");
  let spec: XrayRoots = XrayRoots {
    asset: Some(first.join("configs/system.ltx")),
    roots: vec![XrayRoot::new(first, XrayMountMode::Directory)],
  };

  let plan: XraySearchPlan = spec.to_search_plan().expect("plan");

  assert!(!plan.is_empty());
}

#[test]
fn opens_what_it_planned() {
  let first: PathBuf = root("open", r"configs\system.ltx");
  let spec: XrayRoots = XrayRoots::one(first, XrayMountMode::Directory);

  assert!(open(&spec).read(r"configs\system.ltx").is_ok());
}

#[cfg(any(target_os = "linux", windows))]
#[test]
fn opens_a_non_unicode_root_without_changing_its_address() {
  let base: PathBuf = build_absolute_generated_test_resource_path("xray_roots/non-unicode-address");
  let root: PathBuf = base.join(non_unicode_component());
  let asset: PathBuf = root.join("configs/system.ltx");

  fs::create_dir_all(asset.parent().expect("entry parent")).expect("non-Unicode root directory");
  fs::write(&asset, b"payload").expect("root file");

  let spec: XrayRoots = XrayRoots::one(root, XrayMountMode::Directory);

  assert_eq!(
    open(&spec)
      .read(r"configs\system.ltx")
      .expect("the declared root remains addressable"),
    b"payload"
  );

  // Not left behind: the CI cache archives `target/` with tar, which cannot read a name that is not valid Unicode.
  fs::remove_dir_all(&base).expect("non-Unicode root removal");
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

#[test]
fn lists_each_installation_its_roots_sit_in_once_asset_first() {
  let first: PathBuf = root("installations/first", "fsgame.ltx");
  let second: PathBuf = root("installations/second", "fsgame.ltx");
  let bare: PathBuf = root("installations/bare", "configs/system.ltx");

  fs::create_dir_all(first.join("gamedata")).expect("gamedata");

  let roots: XrayRoots = XrayRoots {
    asset: Some(second.join("gamedata/meshes/a.ogf")),
    roots: vec![
      XrayRoot::new(first.join("gamedata"), XrayMountMode::Auto),
      XrayRoot::new(bare, XrayMountMode::Auto),
      XrayRoot::new(second.clone(), XrayMountMode::Auto),
    ],
  };

  assert_eq!(roots.list_installations(), vec![second, first]);
}

#[test]
fn lists_an_enhanced_edition_installation_by_its_game_named_fsgame() {
  let installation: PathBuf = root("installations/enhanced_edition", "fsgame_cs.ltx");

  fs::create_dir_all(installation.join("gamedata/textures")).expect("gamedata");

  let roots: XrayRoots = XrayRoots::one(installation.join("gamedata/textures"), XrayMountMode::Auto);

  assert_eq!(roots.list_installations(), vec![installation]);
}
