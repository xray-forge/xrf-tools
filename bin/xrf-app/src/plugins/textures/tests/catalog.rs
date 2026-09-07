//! Pins what a listing holds - the fold onto references, the loose directory's own addressing - and what the sweep
//! makes of each descriptor it reads.

use std::fs;
use std::path::PathBuf;

use xrf_db::{ThmBumpMode, ThmTextureFlag, ThmTextureType};
use xrf_material::fixtures::{FixtureTree, ThmFixture};
use xrf_vfs::{XrayMountId, XrayMountMode, XrayRoots, XrayVfs};

use crate::plugins::textures::catalog::{TextureCatalog, TextureCatalogMode, TextureEntry, TextureRole};
use crate::plugins::textures::summary::{TextureBadges, TextureMaterialSummary};
use crate::plugins::textures::tests::fixtures::{
  BASE, BUMP, COMPANION, bumped_tree, catalog, entry, file_source, loose_directory, probe_over, summary, sweep,
  to_dds_bytes,
};

#[test]
fn a_texture_and_its_descriptor_fold_onto_one_entry_by_reference() {
  let tree: FixtureTree = FixtureTree::new("textures_fold")
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
  let tree: FixtureTree = bumped_tree("roles");
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
  let tree: FixtureTree = FixtureTree::new("textures_orphan").with_descriptor(BASE, &ThmFixture::image());
  let catalog: TextureCatalog = catalog(&tree);
  let entry: &TextureEntry = entry(&catalog, BASE);

  assert!(entry.texture.is_none());
  assert!(entry.descriptor.is_some());
}

#[test]
fn a_texture_outside_the_textures_directory_is_counted_and_left_out() {
  let tree: FixtureTree = FixtureTree::new("textures_outside").with_texture(BASE);
  let lightmap: PathBuf = tree.root().join("levels").join("l01_escape").join("lmap#0_1.dds");

  fs::create_dir_all(lightmap.parent().expect("lightmap sits in a directory")).expect("level directory");
  fs::write(&lightmap, b"lightmap").expect("lightmap written");

  let catalog: TextureCatalog = catalog(&tree);

  assert_eq!(catalog.entries.len(), 1);
  assert_eq!(catalog.outside_textures_count, 1);
}

#[test]
fn entries_come_back_in_reference_order() {
  let tree: FixtureTree = FixtureTree::new("textures_order")
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
  let tree: FixtureTree = FixtureTree::new("textures_degraded")
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
  let tree: FixtureTree = FixtureTree::new("textures_skipped")
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
  let tree: FixtureTree = FixtureTree::new("textures_detail")
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
  let tree: FixtureTree = FixtureTree::new("textures_unreadable")
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
  let tree: FixtureTree = bumped_tree("coverage")
    .with_texture("wpn\\wpn_ak74")
    .with_descriptor("act\\act_stalker", &ThmFixture::image());
  let mut references: Vec<String> = sweep(&tree).into_iter().map(|summary| summary.reference).collect();

  references.sort();

  // Two descriptors, three textures without one: a texture with no descriptor has nothing to sweep.
  assert_eq!(references, vec![String::from("act\\act_stalker"), String::from(BASE)]);
}
