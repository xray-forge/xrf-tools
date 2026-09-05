//! The seven declaration states, one test each, so a change that collapses two of them fails by name.

use xrf_db::{ThmBumpChunk, ThmTextureTypeChunk};
use xrf_vfs::{XrayMountId, XrayProbe, XrayVfs};

use crate::fixtures::{ThmFixture, ThmFixtureTree};
use crate::tests::material_probe::{BASE, BUMP, COMPANION, describe, probe_over, used_bump};
use crate::{XrayBumpMode, XrayBumpOutcome, XrayMaterialDeclaration, XrayMaterialDescriptor, XrayMaterialResolver};

#[test]
fn a_texture_without_a_descriptor_is_undeclared_and_flat() {
  let descriptor: XrayMaterialDescriptor = describe(&ThmFixtureTree::new("no_descriptor").with_texture(BASE));

  assert_eq!(descriptor, XrayMaterialDescriptor::undeclared());
}

#[test]
fn a_descriptor_that_does_not_parse_is_unreadable() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("unreadable")
      .with_texture(BASE)
      .with_unreadable_descriptor(BASE),
  );

  assert!(
    matches!(descriptor.declaration, XrayMaterialDeclaration::Unreadable { .. }),
    "{:?}",
    descriptor.declaration
  );
  assert!(descriptor.descriptor.is_some(), "the file that failed is named");
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Flat);
}

#[test]
fn a_bump_map_typed_descriptor_is_skipped_whole_however_complete_its_declaration() {
  // The case a hex editor cannot show: everything in the bump chunk is right, and the engine never reads it.
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("type_disqualified")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_descriptor(BASE, &used_bump().with_texture_type(ThmTextureTypeChunk::BUMP_MAP)),
  );

  assert_eq!(
    descriptor.declaration,
    XrayMaterialDeclaration::TypeDisqualified {
      texture_type: ThmTextureTypeChunk::BUMP_MAP,
      label: String::from("Bump Map"),
      declared_bump: Some(String::from(BUMP)),
    }
  );
  assert_eq!(descriptor.bump, None);
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Flat);
}

#[test]
fn a_descriptor_without_a_type_chunk_is_an_image_and_qualifies() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("no_type_chunk")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_descriptor(BASE, &used_bump().without_texture_type()),
  );

  assert_eq!(descriptor.outcome, XrayBumpOutcome::Bumped);
}

#[test]
fn a_descriptor_without_a_bump_chunk_declares_nothing() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("no_bump_chunk")
      .with_texture(BASE)
      .with_descriptor(BASE, &ThmFixture::image().without_bump()),
  );

  assert_eq!(descriptor.declaration, XrayMaterialDeclaration::NoBumpChunk);
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Flat);
}

#[test]
fn a_disabled_mode_with_a_name_is_disabled_not_declared() {
  // The SDK leaves the name behind when an author turns the mode off; the engine reads the mode and never the name.
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("disabled")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_NONE, BUMP)),
  );

  assert_eq!(
    descriptor.declaration,
    XrayMaterialDeclaration::Disabled {
      mode: ThmBumpChunk::MODE_NONE
    }
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Flat);
}

#[test]
fn the_reserved_mode_is_disabled_as_the_engine_clamps_it() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("reserved")
      .with_texture(BASE)
      .with_descriptor(BASE, &ThmFixture::image().with_bump(ThmBumpChunk::MODE_RESERVED, BUMP)),
  );

  assert_eq!(
    descriptor.declaration,
    XrayMaterialDeclaration::Disabled {
      mode: ThmBumpChunk::MODE_RESERVED
    }
  );
}

#[test]
fn a_used_mode_with_an_empty_name_is_flat() {
  let descriptor: XrayMaterialDescriptor =
    describe(&ThmFixtureTree::new("empty_name").with_texture(BASE).with_descriptor(
      BASE,
      &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE_PARALLAX, ""),
    ));

  assert_eq!(
    descriptor.declaration,
    XrayMaterialDeclaration::EmptyName {
      mode: XrayBumpMode::Parallax
    }
  );
  assert_eq!(descriptor.bump, None);
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Flat);
}

#[test]
fn a_rejected_reference_is_undeclared() {
  let descriptor: XrayMaterialDescriptor = {
    let tree: ThmFixtureTree = ThmFixtureTree::new("rejected").with_texture(BASE);
    let mut vfs: XrayVfs = XrayVfs::new();
    let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");

    XrayMaterialResolver::describe_texture(&probe_over(&vfs, id), "..\\..\\outside")
  };

  assert_eq!(descriptor, XrayMaterialDescriptor::undeclared());
}

#[test]
fn describing_a_located_descriptor_answers_the_same_as_describing_its_texture() {
  let tree: ThmFixtureTree = ThmFixtureTree::new("two_doors")
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(BASE, &used_bump());
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe = probe_over(&vfs, id);

  let located = probe
    .find("textures\\act\\act_stalker.thm")
    .expect("path is valid")
    .get_asset()
    .cloned()
    .expect("descriptor is located");

  assert_eq!(
    XrayMaterialResolver::describe_descriptor(&probe, &located),
    XrayMaterialResolver::describe_texture(&probe, BASE)
  );
}

#[test]
fn a_textures_ltx_in_the_roots_is_named_and_otherwise_unread() {
  // The second declaration source this crate does not read. Naming it is what lets a consumer say the answer may be
  // incomplete on exactly the roots where it may be.
  let tree: ThmFixtureTree = ThmFixtureTree::new("textures_ltx").with_texture(BASE);

  std::fs::write(tree.root().join("textures").join("textures.ltx"), "[specification]\n").expect("ltx written");

  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe = probe_over(&vfs, id);

  assert_eq!(
    XrayMaterialResolver::find_textures_ltx(&probe).map(|asset| asset.get_logical_path().to_string()),
    Some(String::from(XrayMaterialResolver::TEXTURES_LTX_LOGICAL_PATH))
  );
  assert_eq!(
    XrayMaterialResolver::describe_texture(&probe, BASE),
    XrayMaterialDescriptor::undeclared()
  );

  let bare: ThmFixtureTree = ThmFixtureTree::new("no_textures_ltx").with_texture(BASE);
  let mut bare_vfs: XrayVfs = XrayVfs::new();
  let bare_id: XrayMountId = bare_vfs.mount_directory("", bare.root()).expect("tree mounts");

  assert_eq!(
    XrayMaterialResolver::find_textures_ltx(&probe_over(&bare_vfs, bare_id)),
    None
  );
}
