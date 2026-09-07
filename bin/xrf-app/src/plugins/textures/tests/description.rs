//! Pins what describing one texture answers, through the roots that name it and from the path of one that no tree can.
use crate::plugins::textures::source::TextureSource;

use std::path::PathBuf;

use xrf_material::XrayMaterialDescriptor;
use xrf_material::fixtures::{FixtureTree, ThmFixture};
use xrf_vfs::{XrayProbe, XrayRoots, XrayVfs};

use crate::plugins::textures::description::TextureDescription;
use crate::plugins::textures::tests::fixtures::{
  BASE, BUMP, COMPANION, bumped_tree, file_source, loose_directory, mount, probe_over, roots_of, to_dds_bytes,
};

#[test]
fn a_description_carries_the_texture_the_material_and_both_bound_halves() {
  let tree: FixtureTree = bumped_tree("description");
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
  let tree: FixtureTree = FixtureTree::new("textures_describe_orphan").with_descriptor(BASE, &ThmFixture::image());
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
fn a_texture_outside_every_root_is_described_from_its_own_path() {
  // The case the reference machinery cannot answer: no ancestor named `textures`, so no engine reference exists and
  // nothing can be resolved against a tree that is not there. The file is still a texture somebody wants to open.
  let root: PathBuf = loose_directory("plain");
  let texture: PathBuf = root.join("wall.dds");

  std::fs::write(&texture, to_dds_bytes(8)).expect("texture is writable");

  let (vfs, id) = mount(&FixtureTree::new("textures_standalone_roots"));
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

  let (vfs, id) = mount(&FixtureTree::new("textures_standalone_descriptor_roots"));
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

  let (vfs, id) = mount(&FixtureTree::new("textures_standalone_thm_roots"));
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
fn a_standalone_description_is_readable_back_through_the_roots_it_answers() {
  // The description is not the end of the story: the preview decodes the texture and the lit surface uploads it, and
  // both ask for bytes by logical path through these roots. A file no tree can place answers to its own name in its
  // own folder, so that folder has to be one of them - without it the panels read a texture that resolves nowhere and
  // the screen says the picture is unavailable while the descriptor beside it renders perfectly.
  let root: PathBuf = loose_directory("readable_back");
  let texture: PathBuf = root.join("wall.dds");

  std::fs::write(&texture, to_dds_bytes(8)).expect("texture is writable");

  let (vfs, id) = mount(&FixtureTree::new("textures_readable_back_roots"));
  let description: TextureDescription = TextureDescription::describe(
    &probe_over(&vfs, id),
    file_source(texture.clone()),
    XrayRoots::default(),
  )
  .expect("a loose texture is described");

  let logical_path: String = description
    .texture
    .as_ref()
    .expect("the file is located")
    .get_logical_path()
    .as_str()
    .to_owned();

  assert_eq!(logical_path, "wall.dds");
  assert_eq!(
    description.roots.roots.first().map(|it| it.path.as_path()),
    Some(root.as_path()),
    "expect the file's own folder searched first, so the name it answers to is the one it was described under"
  );

  // What the preview does, over the roots the description came back with.
  let mut reader: XrayVfs = XrayVfs::new();
  let steps = description
    .roots
    .to_probe_plan()
    .expect("the answered roots plan")
    .mount_into(&mut reader)
    .expect("the answered roots mount");

  assert!(
    reader
      .probe()
      .with_steps(steps)
      .find(&logical_path)
      .expect("lookup")
      .get_asset()
      .is_some(),
    "expect the texture to resolve back through the roots its own description answered with"
  );
}
