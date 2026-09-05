//! Pins the catalog fold, the sweep's badges, and how a source names its texture, over descriptor trees built from
//! `xrf-material`'s own fixtures.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_db::{ThmBumpChunk, ThmTextureParamChunk, ThmTextureTypeChunk};
use xrf_material::fixtures::{ThmFixture, ThmFixtureTree};
use xrf_vfs::{XrayAssetType, XrayLookupScope, XrayMountId, XrayMountMode, XrayProbe, XrayRoots, XrayVfs};

use crate::plugins::textures::catalog::{TextureCatalog, TextureEntry, TextureRole};
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

  TextureCatalog::list(&probe_over(&vfs, id), roots_of(tree))
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
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, BUMP))
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
    .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, BUMP));
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
        .with_texture_type(ThmTextureTypeChunk::BUMP_MAP)
        .with_bump(ThmBumpChunk::MODE_USE, BUMP),
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
      &ThmFixture::image().with_detail(
        "detail\\detail_grnd_grass",
        1.0,
        ThmTextureParamChunk::FLAG_DIFFUSE_DETAIL,
      ),
    )
    .with_texture("dead")
    .with_descriptor(
      "dead",
      &ThmFixture::image().with_detail("detail\\detail_grnd_grass", 1.0, 0),
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
  assert_eq!(description.material.declared_bump_pair(), Some((BUMP, COMPANION)));
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
  assert!(description.material.descriptor.is_some());
}

#[test]
fn a_file_source_is_named_inside_the_root_the_vfs_implies_for_it() {
  let tree: ThmFixtureTree = implied_root_tree("file_source");
  let texture: PathBuf = tree.root().join("textures").join("ston").join("ston_beton05.dds");
  let descriptor: PathBuf = tree.root().join("textures").join("ston").join("ston_beton05.thm");

  assert_eq!(file_source(texture.clone()).to_reference().as_deref(), Ok(BASE));
  assert_eq!(
    file_source(descriptor).to_reference().as_deref(),
    Ok(BASE),
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
    Ok(String::from(BASE)),
    "a textures directory with no meshes beside it still names the files under it"
  );
  assert!(
    file_source(rooted.root().join("meshes").join("ston_beton05.dds"))
      .to_reference()
      .is_err(),
    "a texture file outside the textures directory is named by no reference"
  );
  assert!(
    file_source(Path::new("C:\\loose\\ston_beton05.dds").to_path_buf())
      .to_reference()
      .is_err()
  );
}
