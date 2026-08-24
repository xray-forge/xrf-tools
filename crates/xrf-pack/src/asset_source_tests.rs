//! Mounts a freshly packed volume set as an asset source.
//!
//! Packing and reading back is the only honest check: an archive source is correct when it answers for volumes the packer
//! actually wrote, in the name form the header actually stores.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::XrayArchiveSource;
use xrf_vfs::{
  XrayAssetContainer, XrayAssetSource, XrayAssetType, XrayLookupScope, XrayMountPlan, XrayProbe, XrayProbePlan,
  XrayProbeStep, XraySourceKind, XrayVfs,
};

use crate::pack::archive_pack_config::{ArchivePackConfig, ArchivePackDirectory};
use crate::pack::archive_packer::ArchivePacker;

const TEXTURE: &[u8] = &[0x44, 0x44, 0x53, 0x20, 0x01, 0x02, 0x03, 0xfe];
const CONFIG: &[u8] = b"[section]\nvalue = 1\n";

/// Packs a source tree into volumes and mounts the result.
fn mount(scope: &str, files: &[(&str, &[u8])]) -> XrayArchiveSource {
  let source: PathBuf = build_absolute_generated_test_resource_path(&format!("archive_asset_source/{scope}/gamedata"));
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("archive_asset_source/{scope}/db"));

  let _ = fs::remove_dir_all(&source);
  let _ = fs::remove_dir_all(&destination);

  for (name, contents) in files {
    let path: PathBuf = source.join(name.replace('\\', "/"));

    fs::create_dir_all(path.parent().expect("entry parent")).expect("source directory");
    fs::write(&path, contents).expect("source file");
  }

  let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, "packed");

  config.include_directories = vec![ArchivePackDirectory {
    is_recursive: true,
    path: String::new(),
  }];

  ArchivePacker::pack(&config).expect("archive packs");

  XrayArchiveSource::read(&destination).expect("volume set mounts")
}

#[test]
fn reports_itself_as_a_read_only_archive() {
  let source: XrayArchiveSource = mount("read_only", &[("textures\\wpn\\wpn_ak74.dds", TEXTURE)]);

  assert_eq!(source.get_kind(), XraySourceKind::Archive);
  assert!(!source.is_writable());
  assert!(source.write("textures\\wpn\\wpn_ak74.dds", TEXTURE).is_err());
}

#[test]
fn contains_and_reads_a_packed_entry_by_logical_path() {
  let source: XrayArchiveSource = mount("reads", &[("textures\\wpn\\wpn_ak74.dds", TEXTURE)]);

  assert!(source.contains("textures\\wpn\\wpn_ak74.dds"));
  assert!(!source.contains("textures\\wpn\\wpn_val.dds"));
  assert_eq!(source.read("textures\\wpn\\wpn_ak74.dds").unwrap(), TEXTURE);
}

#[test]
fn locates_an_entry_in_its_volume_set_rather_than_on_disk() {
  // The container is the whole reason an archived asset cannot be handed to `fs::read`.
  let source: XrayArchiveSource = mount("locates", &[("textures\\wpn\\wpn_ak74.dds", TEXTURE)]);

  assert!(matches!(
    source.locate("textures\\wpn\\wpn_ak74.dds"),
    Some(XrayAssetContainer::Archive { .. })
  ));
  assert_eq!(source.locate("textures\\wpn\\wpn_val.dds"), None);
}

#[test]
fn enumerates_entries_and_narrows_by_prefix() {
  let source: XrayArchiveSource = mount(
    "enumerates",
    &[
      ("textures\\wpn\\wpn_ak74.dds", TEXTURE),
      ("configs\\system.ltx", CONFIG),
      ("configs\\weapons\\ak74.ltx", CONFIG),
    ],
  );

  // Only files. A volume also records the directories it contains, and those must not surface as assets.
  assert_eq!(source.list_entries(None).count(), 3);
  assert_eq!(source.list_entries(Some("configs")).count(), 2);
  assert!(!source.contains("configs"), "a directory entry is not an asset");
  assert!(!source.contains("textures\\wpn"));
}

#[test]
fn resolves_a_texture_reference_once_mounted_in_a_vfs() {
  // What the visuals viewer will do against a real install: the reference completes to a logical path and the archive
  // answers it, with no filesystem path anywhere in the chain.
  let source: XrayArchiveSource = mount("vfs", &[("textures\\wpn\\wpn_ak74.dds", TEXTURE)]);

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount("", Box::new(source)).expect("archive mounts");

  let scope: XrayLookupScope = XrayLookupScope::all();
  let location = vfs
    .scoped(&scope)
    .dds_texture("wpn\\wpn_ak74")
    .expect("lookup succeeds")
    .expect("texture resolves");

  assert_eq!(location.get_logical_path().as_str(), "textures\\wpn\\wpn_ak74.dds");
  assert_eq!(location.to_physical_path(), None);
  assert_eq!(
    vfs.scoped(&scope).read_bytes("textures\\wpn\\wpn_ak74.dds").unwrap(),
    TEXTURE
  );
}

#[test]
fn a_loose_file_wins_over_the_same_name_in_an_archive() {
  // The rule fsgame declares by listing db before gamedata. Mount order carries it.
  let archived: XrayArchiveSource = mount("override", &[("textures\\wpn\\wpn_ak74.dds", TEXTURE)]);

  let loose: PathBuf = build_absolute_generated_test_resource_path("archive_asset_source/override/loose");

  let _ = fs::remove_dir_all(&loose);

  fs::create_dir_all(loose.join("textures/wpn")).expect("loose directory");
  fs::write(loose.join("textures/wpn/wpn_ak74.dds"), b"loose").expect("loose file");

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &loose).expect("directory mounts");
  vfs.mount("", Box::new(archived)).expect("archive mounts");

  let scope: XrayLookupScope = XrayLookupScope::all();

  assert_eq!(
    vfs.scoped(&scope).read_bytes("textures\\wpn\\wpn_ak74.dds").unwrap(),
    b"loose"
  );
  assert_eq!(
    vfs
      .scoped(&scope)
      .find_all("textures\\wpn\\wpn_ak74.dds")
      .unwrap()
      .len(),
    2,
    "the archived copy stays reportable behind the override"
  );
}

#[test]
fn a_directory_of_volumes_is_planned_as_an_archive_source() {
  // What a viewer pointed at `<install>\db` needs: the directory is neither an installation nor a loose tree, and
  // mounting it as the latter answers for `packed.db0` instead of for the assets inside it.
  mount("planned", &[("meshes\\wpn\\wpn_ak74.ogf", TEXTURE)]);

  let volumes: PathBuf = build_absolute_generated_test_resource_path("archive_asset_source/planned/db");

  assert!(XrayMountPlan::holds_volumes(&volumes), "the packer wrote volumes there");

  let mut vfs: XrayVfs = XrayVfs::new();
  let steps: Vec<XrayProbeStep> = XrayProbePlan::new()
    .with_root("browsed root", &volumes)
    .expect("volumes plan")
    .mount_into(&mut vfs)
    .expect("volumes mount");

  let probe: XrayProbe = vfs.probe().with_steps(steps);
  let listed: Vec<String> = probe
    .list_assets_of_type(XrayAssetType::Ogf)
    .into_iter()
    .map(|asset| asset.get_logical_path().as_str().to_string())
    .collect();

  assert_eq!(
    listed,
    ["meshes\\wpn\\wpn_ak74.ogf"],
    "the volume's assets are browsable"
  );
  assert_eq!(
    probe
      .resolve(XrayAssetType::Ogf, "wpn\\wpn_ak74")
      .expect("lookup succeeds")
      .get_asset()
      .and_then(|asset| asset.to_physical_path()),
    None,
    "an archived model has no filesystem path, which is why it is addressed logically"
  );
}
