use std::path::PathBuf;

use crate::vfs::tests::fake_source::{FakeArchiveSource, directory};
use crate::{XrayAssetType, XrayLookupScope, XrayMountId, XrayProbe, XrayResolution, XrayVfs};

/// Two directory mounts, returned with the ids a scope selects them by.
fn two_roots(case: &str, first: &[&str], second: &[&str]) -> (XrayVfs, XrayMountId, XrayMountId, PathBuf, PathBuf) {
  let near: PathBuf = directory(&format!("probe_{case}_near"), first);
  let far: PathBuf = directory(&format!("probe_{case}_far"), second);

  let mut vfs: XrayVfs = XrayVfs::new();

  let near_id: XrayMountId = vfs.mount_directory("", &near).expect("near root mounts");
  let far_id: XrayMountId = vfs.mount_directory("", &far).expect("far root mounts");

  (vfs, near_id, far_id, near, far)
}

#[test]
fn resolves_from_the_first_step_holding_the_reference() {
  let (vfs, near_id, far_id, near, _) = two_roots(
    "first_step",
    &["textures/wpn/wpn_ak74.dds"],
    &["textures/wpn/wpn_ak74.dds", "textures/wpn/wpn_abakan.dds"],
  );

  let resolution: XrayResolution = vfs
    .probe()
    .with_step("near", XrayLookupScope::only([near_id]))
    .with_step("far", XrayLookupScope::only([far_id]))
    .resolve(XrayAssetType::Dds, "wpn\\wpn_ak74")
    .expect("lookup succeeds");

  assert_eq!(resolution.get_step(), Some("near"));
  assert_eq!(
    resolution.get_asset().and_then(|it| it.get_root()),
    Some(near.as_path())
  );
}

#[test]
fn falls_through_to_a_later_step_when_an_earlier_one_does_not_hold_the_reference() {
  let (vfs, near_id, far_id, _, far) = two_roots(
    "fall_through",
    &["textures/wpn/wpn_ak74.dds"],
    &["textures/wpn/wpn_abakan.dds"],
  );

  let resolution: XrayResolution = vfs
    .probe()
    .with_step("near", XrayLookupScope::only([near_id]))
    .with_step("far", XrayLookupScope::only([far_id]))
    .resolve(XrayAssetType::Dds, "wpn\\wpn_abakan")
    .expect("lookup succeeds");

  assert_eq!(resolution.get_step(), Some("far"));
  assert_eq!(resolution.get_asset().and_then(|it| it.get_root()), Some(far.as_path()));
}

#[test]
fn a_step_selecting_no_mount_does_not_end_the_search() {
  let (vfs, _, far_id, _, far) = two_roots(
    "no_mount",
    &["textures/wpn/wpn_ak74.dds"],
    &["textures/wpn/wpn_abakan.dds"],
  );

  // An unconfigured root is the ordinary case for a probe assembled from optional sources, and it must not shadow the
  // configured step behind it.
  let resolution: XrayResolution = vfs
    .probe()
    .with_step("unconfigured", XrayLookupScope::only([]))
    .with_step("far", XrayLookupScope::only([far_id]))
    .resolve(XrayAssetType::Dds, "wpn\\wpn_abakan")
    .expect("lookup succeeds");

  assert_eq!(resolution.get_step(), Some("far"));
  assert_eq!(resolution.get_asset().and_then(|it| it.get_root()), Some(far.as_path()));
}

#[test]
fn reports_no_scope_when_nothing_is_searchable() {
  let vfs: XrayVfs = XrayVfs::new();
  let probe: XrayProbe = vfs.probe().with_step("nothing", XrayLookupScope::all());

  assert!(probe.is_empty(), "a step over an empty vfs searches nothing");
  assert_eq!(
    probe
      .resolve(XrayAssetType::Dds, "wpn\\wpn_ak74")
      .expect("lookup succeeds"),
    XrayResolution::NoScope
  );
}

#[test]
fn reports_every_searched_root_when_nothing_resolves() {
  let (vfs, near_id, far_id, near, far) = two_roots(
    "missing",
    &["textures/wpn/wpn_ak74.dds"],
    &["textures/wpn/wpn_abakan.dds"],
  );

  let resolution: XrayResolution = vfs
    .probe()
    .with_step("near", XrayLookupScope::only([near_id]))
    .with_step("far", XrayLookupScope::only([far_id]))
    .resolve(XrayAssetType::Dds, "wpn\\wpn_absent")
    .expect("lookup succeeds");

  assert_eq!(
    resolution,
    XrayResolution::Missing {
      roots: vec![near.display().to_string(), far.display().to_string()]
    }
  );
}

#[test]
fn substitutes_a_fallback_reference_and_names_it() {
  let (vfs, near_id, far_id, _, far) = two_roots(
    "substituted",
    &["textures/wpn/wpn_ak74.dds"],
    &["textures/ed/ed_not_existing_texture.dds"],
  );

  let resolution: XrayResolution = vfs
    .probe()
    .with_step("near", XrayLookupScope::only([near_id]))
    .with_step("far", XrayLookupScope::only([far_id]))
    .resolve_with_fallback(XrayAssetType::Dds, "wpn\\wpn_absent", "ed\\ed_not_existing_texture")
    .expect("lookup succeeds");

  assert_eq!(
    resolution,
    XrayResolution::Substituted {
      step: "far".into(),
      fallback: "ed\\ed_not_existing_texture".into(),
      assets: resolution.get_assets().to_vec()
    }
  );
  assert_eq!(resolution.get_asset().and_then(|it| it.get_root()), Some(far.as_path()));
}

#[test]
fn a_present_reference_is_never_substituted() {
  let (vfs, near_id, far_id, near, _) = two_roots(
    "present",
    &["textures/wpn/wpn_ak74.dds"],
    &["textures/ed/ed_not_existing_texture.dds"],
  );

  let resolution: XrayResolution = vfs
    .probe()
    .with_step("near", XrayLookupScope::only([near_id]))
    .with_step("far", XrayLookupScope::only([far_id]))
    .resolve_with_fallback(XrayAssetType::Dds, "wpn\\wpn_ak74", "ed\\ed_not_existing_texture")
    .expect("lookup succeeds");

  assert_eq!(resolution.get_step(), Some("near"));
  assert_eq!(
    resolution.get_asset().and_then(|it| it.get_root()),
    Some(near.as_path())
  );
}

#[test]
fn a_masked_motion_reference_is_one_outcome_holding_every_match() {
  let (vfs, near_id, _, _, _) = two_roots(
    "mask",
    &[
      "meshes/wpn/wpn_ak74_hud.omf",
      "meshes/wpn/wpn_ak74_world.omf",
      "meshes/wpn/wpn_abakan_hud.omf",
    ],
    &[],
  );

  let resolution: XrayResolution = vfs
    .probe()
    .with_step("near", XrayLookupScope::only([near_id]))
    .resolve(XrayAssetType::Omf, "wpn\\wpn_ak74_*.omf")
    .expect("lookup succeeds");

  let mut paths: Vec<&str> = resolution
    .get_assets()
    .iter()
    .map(|it| it.get_logical_path().as_str())
    .collect();

  paths.sort_unstable();

  assert_eq!(
    paths,
    ["meshes\\wpn\\wpn_ak74_hud.omf", "meshes\\wpn\\wpn_ak74_world.omf"]
  );
}

#[test]
fn reads_a_located_asset_through_the_probed_vfs() {
  let mut vfs: XrayVfs = XrayVfs::new();

  let id: XrayMountId = vfs
    .mount(
      "",
      Box::new(FakeArchiveSource::new("volume", &["meshes/wpn/wpn_ak74.ogf"])),
    )
    .expect("archive mounts");

  let probe: XrayProbe = vfs.probe().with_step("volume", XrayLookupScope::only([id]));

  let resolution: XrayResolution = probe
    .resolve(XrayAssetType::Ogf, "wpn\\wpn_ak74")
    .expect("lookup succeeds");

  let asset: &crate::XrayAsset = resolution.get_asset().expect("model resolves");

  assert_eq!(
    probe.read_asset_bytes(asset).expect("bytes are readable"),
    b"volume".to_vec()
  );
}

#[test]
fn finds_an_exact_logical_path_in_step_order() {
  let (vfs, near_id, far_id, near, _) = two_roots(
    "find",
    &["textures/wpn/wpn_ak74.dds"],
    &["textures/wpn/wpn_ak74.dds", "textures/wpn/wpn_abakan.dds"],
  );

  let probe: XrayProbe = vfs
    .probe()
    .with_step("near", XrayLookupScope::only([near_id]))
    .with_step("far", XrayLookupScope::only([far_id]));

  let resolution: XrayResolution = probe.find("textures\\wpn\\wpn_ak74.dds").expect("lookup succeeds");

  assert_eq!(resolution.get_step(), Some("near"));
  assert_eq!(
    resolution.get_asset().and_then(|it| it.get_root()),
    Some(near.as_path())
  );

  assert!(
    matches!(
      probe.find("textures\\wpn\\wpn_absent.dds").expect("lookup succeeds"),
      XrayResolution::Missing { .. }
    ),
    "an exact path that no step holds is missing, not an error"
  );
}

#[test]
fn lists_assets_of_a_kind_once_per_identity_with_the_earlier_step_winning() {
  let (vfs, near_id, far_id, near, far) = two_roots(
    "list",
    &["meshes/wpn/wpn_ak74.ogf", "textures/wpn/wpn_ak74.dds"],
    &["meshes/wpn/wpn_ak74.ogf", "meshes/wpn/wpn_abakan.ogf"],
  );

  let assets: Vec<crate::XrayAsset> = vfs
    .probe()
    .with_step("near", XrayLookupScope::only([near_id]))
    .with_step("far", XrayLookupScope::only([far_id]))
    .list_assets_of_type(XrayAssetType::Ogf);

  let mut listed: Vec<(&str, Option<&std::path::Path>)> = assets
    .iter()
    .map(|it| (it.get_logical_path().as_str(), it.get_root()))
    .collect();

  listed.sort_unstable();

  assert_eq!(
    listed,
    [
      ("meshes\\wpn\\wpn_abakan.ogf", Some(far.as_path())),
      ("meshes\\wpn\\wpn_ak74.ogf", Some(near.as_path())),
    ],
    "the shadowed copy is omitted and the texture is not a model"
  );
}
