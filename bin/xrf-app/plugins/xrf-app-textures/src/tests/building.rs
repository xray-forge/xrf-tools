//! Rebuilding a texture or its bump pair: what the processors make, published through the shared mount.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_app_core::assets::AssetMountState;
use xrf_dds::{DdsFile, DdsMipFilter, Quality};
use xrf_job::JobHandle;
use xrf_material::fixtures::FixtureTree;
use xrf_texture::{
  GenerateBumpFormat, GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult,
};
use xrf_vfs::{XrayRevision, XraySearch};

use crate::edit::TextureStagedWriter;
use crate::tests::fixtures::{BUMP, bumped_tree, roots_of, source_image};

/// The options a pair is generated from for the bump at `bump`, without an extension.
fn bump_options(bump: PathBuf) -> GenerateBumpOptions {
  GenerateBumpOptions {
    job: JobHandle::inert(),
    bump,
    height: source_image(8),
    gloss: GenerateBumpGloss::Constant(GenerateBumpOptions::DEFAULT_GLOSS),
    normal_map: None,
    virtual_height: GenerateBumpOptions::DEFAULT_VIRTUAL_HEIGHT,
    mip_filter: DdsMipFilter::Box,
    format: GenerateBumpFormat::Bc3,
    quality: Quality::Fast,
  }
}

/// Every file in a directory, staged leftovers included.
fn list_names(directory: &Path) -> Vec<String> {
  let mut names: Vec<String> = fs::read_dir(directory)
    .expect("the directory lists")
    .map(|entry| entry.expect("an entry").file_name().to_string_lossy().into_owned())
    .collect();

  names.sort();

  names
}

#[test]
fn a_pair_generated_into_a_mounted_tree_lands_as_one_commit() {
  let tree: FixtureTree = bumped_tree("pair_committed");
  let assets: AssetMountState = AssetMountState::new();
  let search: XraySearch = assets.search(&roots_of(&tree)).expect("the tree mounts");
  let bump: PathBuf = tree.root().join("textures").join(BUMP.replace('\\', "/"));
  let before: Vec<u8> = fs::read(bump.with_extension("dds")).expect("the shipped bump reads");
  let mut writer: TextureStagedWriter = TextureStagedWriter::default();
  let result: GenerateBumpResult =
    GenerateBumpProcessor::generate_into(&bump_options(bump), &mut writer).expect("the pair generates");

  assert_eq!(
    fs::read(&result.bump).expect("the bump reads"),
    before,
    "nothing lands before the publish"
  );

  let written: Vec<String> = writer.publish(&assets).expect("the pair publishes");

  assert_eq!(
    written,
    [
      result.bump.display().to_string(),
      result.companion.display().to_string()
    ]
  );
  assert_ne!(
    assets.get_vfs().get_revision(),
    XrayRevision::MOUNTED,
    "the pair is a commit"
  );
  assert_ne!(fs::read(&result.bump).expect("the bump reads"), before);
  assert!(DdsFile::read_from_path(&result.companion).is_ok());
  assert!(search.holds_file(&result.companion), "the companion is the mounted one");
  assert_eq!(
    list_names(result.bump.parent().expect("the pair sits in a folder"))
      .iter()
      .filter(|name| name.contains(".xrf-"))
      .count(),
    0,
    "nothing staged is left beside the pair"
  );
}

#[test]
fn a_writer_dropped_unpublished_leaves_nothing_on_disk() {
  let tree: FixtureTree = bumped_tree("pair_dropped");
  let folder: PathBuf = tree.root().join("textures/ston");
  let before: Vec<String> = list_names(&folder);
  let mut writer: TextureStagedWriter = TextureStagedWriter::default();

  GenerateBumpProcessor::generate_into(&bump_options(folder.join("ston_beton05_new_bump")), &mut writer)
    .expect("the pair generates");

  assert_ne!(list_names(&folder), before, "the pair is staged");

  drop(writer);

  assert_eq!(list_names(&folder), before);
}
