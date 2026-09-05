//! What a bound pair comes to: the files the renderer binds and the outcome the worse half decides.

use xrf_db::ThmBumpChunk;
use xrf_vfs::XrayResolution;

use crate::fixtures::{ThmFixture, ThmFixtureTree};
use crate::tests::material_probe::{BASE, BUMP, COMPANION, describe, located_path, used_bump};
use crate::{XrayBumpFallback, XrayBumpMode, XrayBumpOutcome, XrayMaterialDeclaration, XrayMaterialDescriptor};

#[test]
fn a_declaration_resolving_both_inputs_is_bumped() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("bumped")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_descriptor(BASE, &used_bump().with_virtual_height(0.02)),
  );

  assert_eq!(
    descriptor.declaration,
    XrayMaterialDeclaration::Declared {
      mode: XrayBumpMode::Use,
      name: String::from(BUMP),
    }
  );

  let bump = descriptor.bump.expect("a declared bump carries its pair");

  assert_eq!(bump.mode, XrayBumpMode::Use);
  assert_eq!(bump.virtual_height, 0.02);
  assert_eq!(bump.bump.reference, BUMP);
  assert_eq!(
    bump.companion.reference, COMPANION,
    "the companion is the name with # appended"
  );
  assert_eq!(
    located_path(&bump.bump.resolution),
    Some("textures\\act\\act_stalker_bump.dds")
  );
  assert_eq!(
    located_path(&bump.companion.resolution),
    Some("textures\\act\\act_stalker_bump#.dds")
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Bumped);
}

#[test]
fn a_missing_companion_alone_makes_the_pair_dummy() {
  // The bump exists and the # does not: the engine binds the real bump beside the dummy companion.
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("companion_missing")
      .with_engine_dummies()
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_descriptor(BASE, &used_bump()),
  );

  let bump = descriptor.bump.expect("declared");

  assert!(matches!(bump.bump.resolution, XrayResolution::Resolved { .. }));
  assert!(
    matches!(
      &bump.companion.resolution,
      XrayResolution::Substituted { fallback, .. } if fallback == XrayBumpFallback::DummyCompanion.reference()
    ),
    "{:?}",
    bump.companion.resolution
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Dummy);
}

#[test]
fn a_missing_name_containing_bump_falls_to_the_dummy_pair() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("dummy")
      .with_engine_dummies()
      .with_texture(BASE)
      .with_descriptor(BASE, &used_bump()),
  );

  let bump = descriptor.bump.expect("declared");

  assert_eq!(
    located_path(&bump.bump.resolution),
    Some("textures\\ed\\ed_dummy_bump.dds")
  );
  assert_eq!(
    located_path(&bump.companion.resolution),
    Some("textures\\ed\\ed_dummy_bump#.dds")
  );
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Dummy);
}

#[test]
fn a_missing_name_without_the_marker_falls_to_the_not_existing_texture() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("missing")
      .with_engine_dummies()
      .with_texture(BASE)
      .with_descriptor(
        BASE,
        &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE, "act\\act_stalker_nm"),
      ),
  );

  let bump = descriptor.bump.expect("declared");

  assert!(
    matches!(
      &bump.bump.resolution,
      XrayResolution::Substituted { fallback, .. } if fallback == XrayBumpFallback::NotExisting.reference()
    ),
    "{:?}",
    bump.bump.resolution
  );
  assert_eq!(bump.companion.reference, "act\\act_stalker_nm#");
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Missing);
}

#[test]
fn a_missing_pair_with_no_dummies_is_missing() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("missing_no_dummies")
      .with_texture(BASE)
      .with_descriptor(BASE, &used_bump()),
  );

  let bump = descriptor.bump.expect("declared");

  assert!(matches!(bump.bump.resolution, XrayResolution::Missing { .. }));
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Missing);
}

#[test]
fn a_parallax_declaration_is_reported_as_parallax() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("parallax")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_descriptor(
        BASE,
        &ThmFixture::image().with_bump(ThmBumpChunk::MODE_USE_PARALLAX, BUMP),
      ),
  );

  assert_eq!(descriptor.bump.expect("declared").mode, XrayBumpMode::Parallax);
  assert_eq!(descriptor.outcome, XrayBumpOutcome::Bumped);
}

#[test]
fn the_outcome_of_a_pair_is_the_worse_input() {
  let resolved: XrayResolution = XrayResolution::Resolved {
    step: String::from("tree"),
    assets: Vec::new(),
  };
  let dummy: XrayResolution = XrayResolution::Substituted {
    step: String::from("tree"),
    fallback: String::from(XrayBumpFallback::DummyBump.reference()),
    assets: Vec::new(),
  };
  let missing: XrayResolution = XrayResolution::Missing { roots: Vec::new() };

  assert_eq!(XrayBumpOutcome::of_pair(&resolved, &resolved), XrayBumpOutcome::Bumped);
  assert_eq!(XrayBumpOutcome::of_pair(&resolved, &dummy), XrayBumpOutcome::Dummy);
  assert_eq!(XrayBumpOutcome::of_pair(&dummy, &missing), XrayBumpOutcome::Missing);
  assert!(XrayBumpOutcome::Dummy.is_degraded());
  assert!(!XrayBumpOutcome::Bumped.is_degraded());
  assert!(!XrayBumpOutcome::Flat.is_bump_path());
}
