//! Pins the catalog fold, the sweep's badges, and how a source names its texture, over descriptor trees built from
//! `xrf-material`'s own fixtures.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_db::{ThmBumpMode, ThmTextureFlag, ThmTextureType};
use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_material::XrayMaterialDescriptor;
use xrf_material::fixtures::{ThmFixture, ThmFixtureTree};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayAssetType, XrayLookupScope, XrayMountId, XrayMountMode, XrayProbe, XrayRoots, XrayVfs};

use crate::plugins::textures::catalog::{TextureCatalog, TextureCatalogMode, TextureEntry, TextureRole};
use crate::plugins::textures::description::TextureDescription;
use crate::plugins::textures::source::TextureSource;
use crate::plugins::textures::summary::{TextureBadges, TextureMaterialSummary};

const BASE: &str = "ston\\ston_beton05";
const BUMP: &str = "ston\\ston_beton05_bump";
const COMPANION: &str = "ston\\ston_beton05_bump#";

/// One mounted tree and a probe naming it.
fn mount(tree: &ThmFixtureTree) -> (XrayVfs, XrayMountId) {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");

  (vfs, id)
}

fn probe_over(vfs: &XrayVfs, id: XrayMountId) -> XrayProbe<'_> {
  vfs.probe().with_step("tree", XrayLookupScope::only([id]))
}

fn roots_of(tree: &ThmFixtureTree) -> XrayRoots {
  XrayRoots::one(tree.root().to_path_buf(), XrayMountMode::Directory)
}

fn catalog(tree: &ThmFixtureTree) -> TextureCatalog {
  let (vfs, id) = mount(tree);

  TextureCatalog::list(&probe_over(&vfs, id), roots_of(tree), TextureCatalogMode::Roots)
}

/// The sweep as the command runs it: over every descriptor the probe lists.
fn sweep(tree: &ThmFixtureTree) -> Vec<TextureMaterialSummary> {
  let (vfs, id) = mount(tree);
  let probe: XrayProbe = probe_over(&vfs, id);

  TextureMaterialSummary::sweep(&probe, &probe.list_assets_of_type(XrayAssetType::Thm))
}

fn entry<'a>(catalog: &'a TextureCatalog, reference: &str) -> &'a TextureEntry {
  catalog
    .entries
    .iter()
    .find(|entry| entry.reference == reference)
    .unwrap_or_else(|| panic!("catalog lists '{reference}'"))
}

fn summary<'a>(summaries: &'a [TextureMaterialSummary], reference: &str) -> &'a TextureMaterialSummary {
  summaries
    .iter()
    .find(|summary| summary.reference == reference)
    .unwrap_or_else(|| panic!("sweep describes '{reference}'"))
}

/// A tree with a declared pair that resolves, which is the shape most of these start from.
fn bumped_tree(case: &str) -> ThmFixtureTree {
  ThmFixtureTree::new(&format!("textures_{case}"))
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpMode::Use, BUMP))
}

/// The fixture tree made an X-Ray root the VFS implies: it holds textures already, so a `meshes` directory completes it.
fn implied_root_tree(case: &str) -> ThmFixtureTree {
  let tree: ThmFixtureTree = ThmFixtureTree::new(&format!("textures_{case}")).with_texture(BASE);

  fs::create_dir_all(tree.root().join("meshes")).expect("meshes directory");

  tree
}

fn file_source(path: PathBuf) -> TextureSource {
  TextureSource::File {
    path: path.display().to_string(),
  }
}

/// A real uncompressed dds, so a standalone description has a header to read rather than a placeholder.
fn to_dds_bytes(size: u32) -> Vec<u8> {
  let base: RgbaImage = RgbaImage::from_fn(size, size, |x, y| Rgba([(x * 8) as u8, (y * 8) as u8, 0, u8::MAX]));

  DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
    .encode(&DdsMipChain::build(&base, DdsMipmaps::Disabled).expect("chain"))
    .expect("encode")
    .write_to_bytes()
    .expect("bytes")
}

/// A scratch directory outside every X-Ray root, which is the case a standalone description exists for.
fn loose_directory(case: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("textures_standalone/{case}"));

  let _ = std::fs::remove_dir_all(&root);
  std::fs::create_dir_all(&root).expect("case directory");

  root
}

#[test]
fn a_texture_and_its_descriptor_fold_onto_one_entry_by_reference() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_fold")
    .with_texture(BASE)
    .with_descriptor(BASE, &ThmFixture::image());
  let catalog: TextureCatalog = catalog(&tree);

  assert_eq!(catalog.entries.len(), 1);

  let entry: &TextureEntry = entry(&catalog, BASE);

  assert_eq!(entry.role, TextureRole::Texture);
  assert_eq!(
    entry.texture.as_ref().map(|asset| asset.get_logical_path().as_str()),
    Some("textures\\ston\\ston_beton05.dds")
  );
  assert_eq!(
    entry.descriptor.as_ref().map(|asset| asset.get_logical_path().as_str()),
    Some("textures\\ston\\ston_beton05.thm")
  );
}

#[test]
fn bump_halves_are_listed_with_their_roles_and_the_declared_name_folds_them() {
  let tree: ThmFixtureTree = bumped_tree("roles");
  let catalog: TextureCatalog = catalog(&tree);

  assert_eq!(entry(&catalog, BASE).role, TextureRole::Texture);
  assert_eq!(entry(&catalog, BUMP).role, TextureRole::Bump);
  assert_eq!(entry(&catalog, COMPANION).role, TextureRole::BumpCompanion);

  // The sweep names the bump the base declares, which is what folds the two halves under it in a tree.
  let summaries: Vec<TextureMaterialSummary> = sweep(&tree);

  assert_eq!(
    summary(&summaries, BASE)
      .bump
      .as_ref()
      .map(|pair| (pair.bump.as_str(), pair.companion.as_str())),
    Some((BUMP, COMPANION))
  );
}

#[test]
fn a_descriptor_without_a_texture_is_an_entry_of_its_own() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_orphan").with_descriptor(BASE, &ThmFixture::image());
  let catalog: TextureCatalog = catalog(&tree);
  let entry: &TextureEntry = entry(&catalog, BASE);

  assert!(entry.texture.is_none());
  assert!(entry.descriptor.is_some());
}

#[test]
fn a_texture_outside_the_textures_directory_is_counted_and_left_out() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_outside").with_texture(BASE);
  let lightmap: PathBuf = tree.root().join("levels").join("l01_escape").join("lmap#0_1.dds");

  fs::create_dir_all(lightmap.parent().expect("lightmap sits in a directory")).expect("level directory");
  fs::write(&lightmap, b"lightmap").expect("lightmap written");

  let catalog: TextureCatalog = catalog(&tree);

  assert_eq!(catalog.entries.len(), 1);
  assert_eq!(catalog.outside_textures_count, 1);
}

#[test]
fn entries_come_back_in_reference_order() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_order")
    .with_texture("wpn\\wpn_ak74")
    .with_texture("act\\act_stalker")
    .with_texture("ston\\ston_beton05");
  let catalog: TextureCatalog = catalog(&tree);
  let references: Vec<&str> = catalog.entries.iter().map(|entry| entry.reference.as_str()).collect();

  assert_eq!(
    references,
    vec!["act\\act_stalker", "ston\\ston_beton05", "wpn\\wpn_ak74"]
  );
}

#[test]
fn a_resolving_pair_is_bumped_and_nothing_else() {
  let summaries: Vec<TextureMaterialSummary> = sweep(&bumped_tree("bumped"));

  assert_eq!(
    summary(&summaries, BASE).badges,
    TextureBadges {
      is_bumped: true,
      ..TextureBadges::default()
    }
  );
}

#[test]
fn a_declared_bump_the_dummy_stands_in_for_is_degraded() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_degraded")
    .with_engine_dummies()
    .with_texture(BASE)
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpMode::Use, BUMP));
  let summaries: Vec<TextureMaterialSummary> = sweep(&tree);

  assert_eq!(
    summary(&summaries, BASE).badges,
    TextureBadges {
      is_degraded: true,
      ..TextureBadges::default()
    }
  );
}

#[test]
fn a_type_the_engine_skips_is_engine_skipped_and_binds_nothing() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_skipped")
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(
      BASE,
      &ThmFixture::image()
        .with_texture_type(ThmTextureType::BumpMap)
        .with_bump(ThmBumpMode::Use, BUMP),
    );
  let summaries: Vec<TextureMaterialSummary> = sweep(&tree);
  let summary: &TextureMaterialSummary = summary(&summaries, BASE);

  assert_eq!(
    summary.badges,
    TextureBadges {
      is_engine_skipped: true,
      ..TextureBadges::default()
    }
  );
  assert_eq!(summary.bump, None);
}

#[test]
fn a_detail_with_a_live_flag_is_detail_associated_and_a_dead_one_is_not() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_detail")
    .with_texture("live")
    .with_descriptor(
      "live",
      &ThmFixture::image().with_detail("detail\\detail_grnd_grass", 1.0, &[ThmTextureFlag::DiffuseDetail]),
    )
    .with_texture("dead")
    .with_descriptor(
      "dead",
      &ThmFixture::image().with_detail("detail\\detail_grnd_grass", 1.0, &[]),
    );
  let summaries: Vec<TextureMaterialSummary> = sweep(&tree);

  assert!(summary(&summaries, "live").badges.is_detail_associated);
  assert!(!summary(&summaries, "dead").badges.is_detail_associated);
}

#[test]
fn a_descriptor_that_does_not_parse_is_unreadable() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_unreadable")
    .with_texture(BASE)
    .with_unreadable_descriptor(BASE);
  let summaries: Vec<TextureMaterialSummary> = sweep(&tree);

  assert_eq!(
    summary(&summaries, BASE).badges,
    TextureBadges {
      is_unreadable: true,
      ..TextureBadges::default()
    }
  );
}

#[test]
fn the_sweep_describes_every_descriptor_and_only_descriptors() {
  let tree: ThmFixtureTree = bumped_tree("coverage")
    .with_texture("wpn\\wpn_ak74")
    .with_descriptor("act\\act_stalker", &ThmFixture::image());
  let mut references: Vec<String> = sweep(&tree).into_iter().map(|summary| summary.reference).collect();

  references.sort();

  // Two descriptors, three textures without one: a texture with no descriptor has nothing to sweep.
  assert_eq!(references, vec![String::from("act\\act_stalker"), String::from(BASE)]);
}

#[test]
fn a_description_carries_the_texture_the_material_and_both_bound_halves() {
  let tree: ThmFixtureTree = bumped_tree("description");
  let (vfs, id) = mount(&tree);
  let description: TextureDescription = TextureDescription::describe(
    &probe_over(&vfs, id),
    TextureSource::Asset {
      reference: String::from(BASE),
    },
    roots_of(&tree),
  )
  .expect("texture is described");

  assert_eq!(description.reference, BASE);
  assert_eq!(
    description
      .texture
      .as_ref()
      .map(|asset| asset.get_logical_path().as_str()),
    Some("textures\\ston\\ston_beton05.dds")
  );
  // Placeholders are not DDS files, so each shape is the byte count alone and no header.
  assert_eq!(description.base.as_ref().map(|base| base.size), Some(BASE.len() as u64));
  assert!(description.base.as_ref().is_some_and(|base| base.shape.is_none()));
  assert_eq!(description.bump.as_ref().map(|bump| bump.size), Some(BUMP.len() as u64));
  assert_eq!(
    description.companion.as_ref().map(|companion| companion.size),
    Some(COMPANION.len() as u64)
  );
  assert_eq!(
    description
      .material
      .as_ref()
      .and_then(XrayMaterialDescriptor::declared_bump_pair),
    Some((BUMP, COMPANION))
  );
}

#[test]
fn a_descriptor_without_a_texture_describes_with_no_base() {
  let tree: ThmFixtureTree =
    ThmFixtureTree::new("textures_describe_orphan").with_descriptor(BASE, &ThmFixture::image());
  let (vfs, id) = mount(&tree);
  let description: TextureDescription = TextureDescription::describe(
    &probe_over(&vfs, id),
    TextureSource::Asset {
      reference: String::from(BASE),
    },
    roots_of(&tree),
  )
  .expect("descriptor is described");

  assert!(description.texture.is_none());
  assert!(description.base.is_none());
  assert!(
    description
      .material
      .is_some_and(|material| material.descriptor.is_some())
  );
}

#[test]
fn a_file_source_is_named_inside_the_root_the_vfs_implies_for_it() {
  let tree: ThmFixtureTree = implied_root_tree("file_source");
  let texture: PathBuf = tree.root().join("textures").join("ston").join("ston_beton05.dds");
  let descriptor: PathBuf = tree.root().join("textures").join("ston").join("ston_beton05.thm");

  assert_eq!(file_source(texture.clone()).to_reference().as_deref(), Some(BASE));
  assert_eq!(
    file_source(descriptor).to_reference().as_deref(),
    Some(BASE),
    "the descriptor beside a texture names the same texture"
  );
  assert_eq!(file_source(texture.clone()).physical_path(), Some(texture.as_path()));
  assert_eq!(
    TextureSource::Asset {
      reference: String::from(BASE),
    }
    .physical_path(),
    None
  );
}

#[test]
fn a_file_outside_any_root_or_outside_textures_names_no_reference() {
  let loose: ThmFixtureTree = ThmFixtureTree::new("textures_file_loose").with_texture(BASE);
  let rooted: ThmFixtureTree = implied_root_tree("file_misplaced");

  assert_eq!(
    file_source(loose.root().join("textures").join("ston").join("ston_beton05.dds")).to_reference(),
    Some(String::from(BASE)),
    "a textures directory with no meshes beside it still names the files under it"
  );
  assert_eq!(
    file_source(rooted.root().join("meshes").join("ston_beton05.dds")).to_reference(),
    None,
    "a texture file outside the textures directory is named by no reference"
  );
  assert_eq!(
    file_source(Path::new("C:\\loose\\ston_beton05.dds").to_path_buf()).to_reference(),
    None,
    "a file under no root at all names none either, and is described from its own path instead"
  );
}

#[test]
fn a_texture_outside_every_root_is_described_from_its_own_path() {
  // The case the reference machinery cannot answer: no ancestor named `textures`, so no engine reference exists and
  // nothing can be resolved against a tree that is not there. The file is still a texture somebody wants to open.
  let root: PathBuf = loose_directory("plain");
  let texture: PathBuf = root.join("wall.dds");

  std::fs::write(&texture, to_dds_bytes(8)).expect("texture is writable");

  let (vfs, id) = mount(&ThmFixtureTree::new("textures_standalone_roots"));
  let description: TextureDescription = TextureDescription::describe(
    &probe_over(&vfs, id),
    file_source(texture.clone()),
    XrayRoots::default(),
  )
  .expect("a loose texture is described");

  assert_eq!(
    description.reference, "wall",
    "expect the file stem where there is no engine reference"
  );
  assert_eq!(
    description.material, None,
    "expect no material: there is no tree to resolve a bump pair or a detail against"
  );
  assert!(description.bump.is_none() && description.companion.is_none());
  assert_eq!(
    description
      .base
      .and_then(|base| base.shape)
      .map(|shape| (shape.width, shape.height)),
    Some((8, 8)),
    "expect the dds header to be read straight off the path"
  );
}

#[test]
fn a_standalone_texture_reads_the_descriptor_beside_it_and_offers_to_author_one() {
  let root: PathBuf = loose_directory("descriptor");
  let texture: PathBuf = root.join("wall.dds");
  let descriptor: PathBuf = root.join("wall.thm");

  std::fs::write(&texture, to_dds_bytes(8)).expect("texture is writable");

  let (vfs, id) = mount(&ThmFixtureTree::new("textures_standalone_descriptor_roots"));
  let probe: XrayProbe = probe_over(&vfs, id);

  // With no `.thm` beside it the form is absent and the editor authors one; the target says so by expecting nothing.
  let authoring: TextureDescription =
    TextureDescription::describe(&probe, file_source(texture.clone()), XrayRoots::default()).expect("described");

  assert!(authoring.form.is_none());
  assert_eq!(
    authoring
      .targets
      .as_ref()
      .and_then(|targets| targets.descriptor.expected),
    None
  );
  assert!(
    authoring
      .targets
      .as_ref()
      .is_some_and(|targets| targets.descriptor.path.ends_with("wall.thm")),
    "expect the descriptor to be the sibling with the extension swapped, which is the engine's own rule"
  );

  // The sibling `.thm` is read straight off disk, because no mount holds it either.
  std::fs::write(&descriptor, ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let described: TextureDescription =
    TextureDescription::describe(&probe, file_source(texture), XrayRoots::default()).expect("described");

  assert!(described.form.is_some());
  assert!(
    described
      .targets
      .is_some_and(|targets| targets.descriptor.expected.is_some())
  );
}

#[test]
fn a_descriptor_opened_on_its_own_outside_a_root_finds_its_texture() {
  // Either half of the pair may be picked, exactly as inside a tree.
  let root: PathBuf = loose_directory("from_thm");

  std::fs::write(root.join("wall.dds"), to_dds_bytes(8)).expect("texture is writable");
  std::fs::write(root.join("wall.thm"), ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let (vfs, id) = mount(&ThmFixtureTree::new("textures_standalone_thm_roots"));
  let description: TextureDescription = TextureDescription::describe(
    &probe_over(&vfs, id),
    file_source(root.join("wall.thm")),
    XrayRoots::default(),
  )
  .expect("described");

  assert_eq!(description.reference, "wall");
  assert!(description.form.is_some());
  assert!(description.base.is_some(), "expect the dds beside the descriptor");
}

#[test]
fn a_plain_directory_lists_every_texture_it_holds_rather_than_none() {
  // The case decision 21 exists for: a folder somebody is authoring in yields no engine reference for anything, so
  // listed as a game tree it is an empty tree with a count beside it. Listed as itself, every file is the point.
  let root: PathBuf = loose_directory("loose_listing");

  std::fs::create_dir_all(root.join("wall")).expect("nested directory");
  std::fs::write(root.join("brick01.dds"), to_dds_bytes(4)).expect("texture is writable");
  std::fs::write(root.join("brick01.thm"), ThmFixture::image().to_bytes()).expect("descriptor is writable");
  std::fs::write(root.join("wall").join("brick01.dds"), to_dds_bytes(4)).expect("texture is writable");

  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs
    .mount_directory("", &root)
    .expect("a plain directory mounts as a root");
  let roots: XrayRoots = XrayRoots::one(root.clone(), XrayMountMode::Directory);

  let listed: TextureCatalog =
    TextureCatalog::list(&probe_over(&vfs, id), roots.clone(), TextureCatalogMode::LooseDirectory);

  assert_eq!(listed.mode, TextureCatalogMode::LooseDirectory);
  assert_eq!(
    listed.entries.len(),
    2,
    "expect both textures listed, and the descriptor folded onto the one it sits beside"
  );
  assert_eq!(
    listed.outside_textures_count, 0,
    "expect nothing counted as outside, because in this mode there is no inside"
  );

  let nested: &TextureEntry = entry(&listed, "wall\\brick01");

  assert_eq!(
    nested.source,
    file_source(root.join("wall").join("brick01.dds")),
    "expect a loose row to be opened by its own file rather than by a reference nothing can resolve"
  );

  let beside: &TextureEntry = entry(&listed, "brick01");

  assert!(
    beside.descriptor.is_some(),
    "expect the .thm beside a texture to fold onto it"
  );

  // Keyed by the path below the root rather than by the stem, or these two would have folded into one row.
  assert_ne!(beside.reference, nested.reference);

  // The same directory listed as a game tree, which is what the mode is choosing between.
  let as_tree: TextureCatalog = TextureCatalog::list(&probe_over(&vfs, id), roots, TextureCatalogMode::Roots);

  assert!(as_tree.entries.is_empty());
  assert_eq!(as_tree.outside_textures_count, 2);
}
