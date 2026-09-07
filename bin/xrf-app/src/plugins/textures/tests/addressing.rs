//! Pins how a texture is named: the reference a file answers to, the two files it is, and which reader reaches it.
use crate::plugins::textures::source::TextureSource;
use xrf_material::fixtures::FixtureTree;

use std::path::{Path, PathBuf};

use xrf_vfs::{XrayAssetType, XrayProbe};

use crate::core::assets::{read_located_asset, read_referenced_asset};
use crate::plugins::textures::files::TextureFiles;
use crate::plugins::textures::tests::fixtures::{
  BASE, file_source, implied_root_tree, loose_directory, mount, probe_over,
};

#[test]
fn a_file_source_is_named_inside_the_root_the_vfs_implies_for_it() {
  let tree: FixtureTree = implied_root_tree("file_source");
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
  let loose: FixtureTree = FixtureTree::new("textures_file_loose").with_texture(BASE);
  let rooted: FixtureTree = implied_root_tree("file_misplaced");

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
fn a_source_naming_the_descriptor_still_answers_the_texture_beside_it() {
  // A person opens a texture by picking either half of the pair, so the file named is not always the file the pixels
  // are in. Anything decoding bytes has to ask for the texture rather than for what was named, or it reads a chunked
  // descriptor as a dds and reports a bad magic number at a person who picked a perfectly ordinary file.
  let root: PathBuf = loose_directory("named_descriptor");
  let texture: PathBuf = root.join("wall.dds");
  let descriptor: PathBuf = root.join("wall.thm");

  assert_eq!(
    file_source(descriptor.clone()).to_texture_path(),
    Some(texture.clone()),
    "expect the descriptor's own path to answer the texture beside it"
  );
  assert_eq!(
    file_source(texture.clone()).to_texture_path(),
    Some(texture.clone()),
    "expect a texture to answer itself"
  );

  // What was named is still what was named: the two questions have two answers.
  assert_eq!(
    file_source(descriptor.clone()).physical_path(),
    Some(descriptor.as_path()),
    "expect the named file to be reported as named, for anything centring roots or saying what was opened"
  );
}

#[test]
fn the_two_files_a_texture_is_are_named_from_either_half() {
  // The rule the whole plugin rests on, pinned at its owner rather than at each of the four surfaces that used to
  // spell it: a description reads both halves, a save writes both, a comparison decodes the texture, and a person
  // opens whichever of the two they picked.
  let root: PathBuf = loose_directory("texture_files");
  let texture: PathBuf = root.join("wall.dds");
  let descriptor: PathBuf = root.join("wall.thm");
  let expected: TextureFiles = TextureFiles {
    texture: texture.clone(),
    descriptor: descriptor.clone(),
  };

  assert_eq!(
    TextureFiles::of(&texture),
    expected,
    "expect a texture to name the pair"
  );
  assert_eq!(
    TextureFiles::of(&descriptor),
    expected,
    "expect a descriptor to name the same pair"
  );

  // A name carrying dots keeps every one of them but the last: `wall.v2` is a stem, not an extension to preserve.
  assert_eq!(
    TextureFiles::of(&root.join("wall.v2.dds")).descriptor,
    root.join("wall.v2.thm"),
    "expect only the extension to be replaced"
  );

  // The companion half of a bump pair is an ordinary texture with a `#` in its name, and it has a descriptor too.
  assert_eq!(
    TextureFiles::of(&root.join("wall_bump#.dds")).descriptor,
    root.join("wall_bump#.thm"),
    "expect the companion's own name to survive being paired"
  );
}

#[test]
fn a_texture_is_read_by_its_reference_and_not_by_its_reference_as_a_path() {
  // The defect this pins broke weighing for every texture that has an engine reference, which is every texture in a
  // game tree. A reference such as `ston\\ston_beton05` and the logical path `textures\\ston\\ston_beton05.dds` are
  // different strings for the same file, and a reader given the wrong one resolves nothing at all - so the reader that
  // takes a reference has to be the one a caller holding a reference reaches for.
  let tree: FixtureTree = FixtureTree::new("textures_read_by_reference").with_texture(BASE);
  let (vfs, id) = mount(&tree);
  let probe: XrayProbe = probe_over(&vfs, id);

  assert!(
    read_referenced_asset(&probe, XrayAssetType::Dds, BASE).is_ok(),
    "expect an engine reference to reach the file it names"
  );
  assert!(
    read_located_asset(&probe, BASE).is_err(),
    "expect a reference read as a logical path to resolve to nothing, which is what made this worth pinning"
  );

  // And the logical path the listing reports is what the path reader wants.
  assert!(read_located_asset(&probe, "textures\\ston\\ston_beton05.dds").is_ok());
}
