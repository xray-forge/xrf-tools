//! Mounts a freshly packed volume set as an asset source.
//!
//! Packing and reading back is the only honest check: an archive source is correct when it answers for volumes the packer
//! actually wrote, in the name form the header actually stores.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::XrayArchiveSource;
use xrf_vfs::{
  XrayAssetContainer, XrayAssetSource, XrayAssetType, XrayCollisionSite, XrayDeclaredRoot, XrayLookupScope,
  XrayMountPlan, XrayPathCollision, XrayProbe, XrayProbePlan, XrayProbeStep, XraySourceKind, XrayVfs,
};

use crate::pack::ArchivePacker;
use crate::pack::config::{ArchivePackConfig, ArchivePackDirectory};

const TEXTURE: &[u8] = &[0x44, 0x44, 0x53, 0x20, 0x01, 0x02, 0x03, 0xfe];
const CONFIG: &[u8] = b"[section]\nvalue = 1\n";
/// Distinct payload for the volume that must win, so a test names which volume answered rather than only that one did.
const PATCHED: &[u8] = b"patched";

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

/// Packs each named tree into its own volume of one destination, in the order given.
///
/// Two spellings of one name cannot share a directory on a case-insensitive filesystem, so they are authored in separate
/// trees and meet only inside the volume set — which is how a case-only duplicate reaches a player's install in the
/// first place, as a patch volume built elsewhere.
fn mount_volumes(scope: &str, volumes: &[(&str, &str, &[u8])]) -> XrayArchiveSource {
  let destination: PathBuf = build_absolute_generated_test_resource_path(&format!("archive_asset_source/{scope}/db"));

  let _ = fs::remove_dir_all(&destination);

  for (volume, name, contents) in volumes {
    let source: PathBuf =
      build_absolute_generated_test_resource_path(&format!("archive_asset_source/{scope}/{volume}"));
    let path: PathBuf = source.join(name.replace('\\', "/"));

    let _ = fs::remove_dir_all(&source);

    fs::create_dir_all(path.parent().expect("entry parent")).expect("source directory");
    fs::write(&path, contents).expect("source file");

    let mut config: ArchivePackConfig = ArchivePackConfig::new(&source, &destination, volume);

    config.include_directories = vec![ArchivePackDirectory {
      is_recursive: true,
      path: String::new(),
    }];

    ArchivePacker::pack(&config).expect("archive packs");
  }

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
      ("configs\\empty.ltx", b""),
    ],
  );

  // Only files. A volume also records the directories it contains, and those must not surface as assets.
  assert_eq!(source.list_entries(None).count(), 4);
  assert_eq!(source.list_entries(Some("configs")).count(), 3);
  assert!(!source.contains("configs"), "a directory entry is not an asset");
  assert!(!source.contains("textures\\wpn"));
  assert!(source.contains("configs\\empty.ltx"), "a zero-byte file is an asset");
  assert_eq!(source.read("configs\\empty.ltx").expect("empty file reads"), b"");
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

#[test]
fn a_case_only_duplicate_across_volumes_is_reported_and_resolved_by_volume_order() {
  let source: XrayArchiveSource = mount_volumes(
    "case_collision",
    &[
      ("base", "textures\\wpn\\wpn_ak74.dds", TEXTURE),
      ("patch", "Textures\\Wpn\\WPN_AK74.DDS", PATCHED),
    ],
  );

  assert_eq!(
    source
      .read("textures\\wpn\\wpn_ak74.dds")
      .expect("the folded identity reads"),
    PATCHED,
    "the later volume answers, as CLocatorAPI::Register resolves it"
  );

  let collisions: &[XrayPathCollision] = source.get_collisions();

  assert_eq!(collisions.len(), 1, "one identity, one report");
  assert_eq!(collisions[0].logical_path.as_str(), "textures\\wpn\\wpn_ak74.dds");
  assert_site(&collisions[0].kept, "patch.db", "Textures\\Wpn\\WPN_AK74.DDS");
  assert_site(&collisions[0].unreachable, "base.db", "textures\\wpn\\wpn_ak74.dds");
}

#[test]
fn an_exact_name_override_across_volumes_is_precedence_rather_than_a_collision() {
  // The documented merge this must not start reporting: two volumes naming one file identically is shadowing, which has
  // a defined winner, not an authoring error with an unreachable loser.
  let source: XrayArchiveSource = mount_volumes(
    "exact_override",
    &[
      ("base", "configs\\system.ltx", CONFIG),
      ("patch", "configs\\system.ltx", PATCHED),
    ],
  );

  assert_eq!(source.read("configs\\system.ltx").expect("reads"), PATCHED);
  assert!(source.get_collisions().is_empty());
}

/// Asserts a collision side names one authored entry of one volume.
fn assert_site(site: &XrayCollisionSite, expected_volume: &str, expected_name: &str) {
  match site {
    XrayCollisionSite::Archived { volume, name } => {
      assert_eq!(volume.file_name().and_then(|name| name.to_str()), Some(expected_volume));
      assert_eq!(name, expected_name, "the authored spelling survives the fold");
    }
    XrayCollisionSite::Loose(path) => panic!("archived entry expected, got loose {}", path.display()),
  }
}

#[test]
fn a_recorded_checksum_is_answered_without_reading_the_payload() {
  // What lets a comparison between two volume sets decide from name tables alone: the packer wrote this checksum and
  // the engine verifies it on every decompression, so nothing has to be decompressed to learn it.
  let source: XrayArchiveSource = mount("recorded_crc", &[("configs\\system.ltx", CONFIG)]);
  let recorded: u32 = source
    .get_recorded_crc("configs\\system.ltx")
    .expect("an archived entry records its checksum");

  assert_eq!(recorded, crc32fast::hash(CONFIG), "it is the checksum of the payload");
  assert!(
    source.get_recorded_crc("configs\\absent.ltx").is_none(),
    "an entry the set does not hold records nothing"
  );
}

#[test]
fn a_loose_source_records_no_checksum_of_its_own() {
  // The default half of the seam. A directory would have to read the file to produce one, so it declines and leaves
  // the caller to decide whether the payload is worth reading.
  let root: PathBuf = build_absolute_generated_test_resource_path("archive_asset_source/loose_crc/gamedata");

  let _ = fs::remove_dir_all(&root);
  fs::create_dir_all(root.join("configs")).expect("source directory");
  fs::write(root.join("configs").join("system.ltx"), CONFIG).expect("source file");

  let vfs: XrayVfs = XrayVfs::from_plan(&XrayMountPlan::root(&root).expect("plan")).expect("mounts");

  assert_eq!(vfs.read_size("configs\\system.ltx"), Some(CONFIG.len() as u64));
  assert!(
    vfs.read_recorded_crc("configs\\system.ltx").is_none(),
    "a loose file records nothing, however readable it is"
  );
}

#[test]
fn a_checksum_is_read_through_the_mount_that_won() {
  // A loose override in front of a volume means the archive's recorded checksum must not answer for it: the winning
  // mount is a directory, which records none.
  let scope: &str = "crc_follows_the_winner";
  let archived: XrayArchiveSource = mount(scope, &[("configs\\system.ltx", CONFIG)]);
  let loose: PathBuf = build_absolute_generated_test_resource_path(&format!("archive_asset_source/{scope}/loose"));

  let _ = fs::remove_dir_all(&loose);
  fs::create_dir_all(loose.join("configs")).expect("override directory");
  fs::write(loose.join("configs").join("system.ltx"), PATCHED).expect("override file");

  let mut vfs: XrayVfs = XrayVfs::new();

  vfs.mount_directory("", &loose).expect("loose mount wins");
  vfs.mount("", Box::new(archived)).expect("archive mount behind it");

  assert_eq!(vfs.read_bytes("configs\\system.ltx").expect("reads"), PATCHED);
  assert!(
    vfs.read_recorded_crc("configs\\system.ltx").is_none(),
    "the archive's checksum describes a payload no lookup reaches"
  );
}

#[test]
fn a_volume_declares_the_root_its_entries_mount_under() {
  // Nothing applies this, so a consumer whose answer depends on where entries land has to ask. Every shipped release
  // declares the gamedata root, and a patch is only sound over volumes that do.
  let source: XrayArchiveSource = mount("declared_roots", &[("configs\\system.ltx", CONFIG)]);
  let declared: Vec<XrayDeclaredRoot> = source.list_declared_roots();

  assert_eq!(declared.len(), 1, "one per volume of the set");
  assert_eq!(
    declared[0].root.to_string_lossy(),
    "gamedata\\",
    "the packer's default header, with its alias stripped"
  );
  assert_eq!(
    declared[0].source.file_name().and_then(|name| name.to_str()),
    Some("packed.db"),
    "each declaration names the volume that made it"
  );
}

#[test]
fn a_loose_source_declares_no_root() {
  // A directory mounts where it was mounted and claims nothing, which is why the default is empty rather than a guess.
  let root: PathBuf = build_absolute_generated_test_resource_path("archive_asset_source/loose_roots/gamedata");

  let _ = fs::remove_dir_all(&root);
  fs::create_dir_all(root.join("configs")).expect("source directory");
  fs::write(root.join("configs").join("system.ltx"), CONFIG).expect("source file");

  let vfs: XrayVfs = XrayVfs::from_plan(&XrayMountPlan::root(&root).expect("plan")).expect("mounts");

  for mount in vfs.get_mounts() {
    assert!(mount.get_source().list_declared_roots().is_empty());
  }
}
