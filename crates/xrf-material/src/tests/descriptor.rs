//! The questions a surface asks of a resolved material, answered here once so no consumer re-derives them.

use xrf_db::{ThmTextureParamChunk, ThmTextureTypeChunk};

use crate::fixtures::{ThmFixture, ThmFixtureTree};
use crate::tests::material_probe::{BASE, BUMP, COMPANION, describe, used_bump};
use crate::{XrayBumpNaming, XrayMaterialDescriptor};

#[test]
fn a_declared_bump_is_named_and_a_flat_material_names_none() {
  let declared: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("predicate_declared")
      .with_texture(BASE)
      .with_texture(BUMP)
      .with_texture(COMPANION)
      .with_descriptor(BASE, &used_bump()),
  );
  let flat: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("predicate_flat")
      .with_texture(BASE)
      .with_descriptor(BASE, &ThmFixture::image()),
  );

  assert_eq!(declared.declared_bump_reference(), Some(BUMP));
  assert_eq!(flat.declared_bump_reference(), None);
  assert!(!declared.is_engine_skipped() && !declared.is_unreadable() && !declared.is_detail_associated());
}

#[test]
fn a_skipped_type_and_an_unreadable_file_are_told_apart() {
  let skipped: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("predicate_skipped")
      .with_texture(BASE)
      .with_descriptor(BASE, &used_bump().with_texture_type(ThmTextureTypeChunk::BUMP_MAP)),
  );
  let unreadable: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("predicate_unreadable")
      .with_texture(BASE)
      .with_unreadable_descriptor(BASE),
  );

  assert!(skipped.is_engine_skipped() && !skipped.is_unreadable());
  assert!(unreadable.is_unreadable() && !unreadable.is_engine_skipped());
  assert_eq!(
    skipped.declared_bump_reference(),
    None,
    "a skipped declaration binds nothing"
  );
}

#[test]
fn a_detail_is_associated_only_when_a_flag_switches_it_on() {
  let live: XrayMaterialDescriptor = describe(&ThmFixtureTree::new("predicate_detail_live").with_descriptor(
    BASE,
    &ThmFixture::image().with_detail("detail\\detail_grnd_grass", 1.0, ThmTextureParamChunk::FLAG_BUMP_DETAIL),
  ));
  let dead: XrayMaterialDescriptor = describe(&ThmFixtureTree::new("predicate_detail_dead").with_descriptor(
    BASE,
    &ThmFixture::image().with_detail("detail\\detail_grnd_grass", 1.0, 0),
  ));

  assert!(live.is_detail_associated());
  assert!(!dead.is_detail_associated(), "a name without a flag is dead authoring");
}

#[test]
fn a_companion_is_the_bump_name_with_the_suffix_the_renderer_appends() {
  assert_eq!(XrayBumpNaming::companion_of(BUMP), COMPANION);
  assert!(XrayBumpNaming::is_companion(COMPANION) && !XrayBumpNaming::is_companion(BUMP));
  assert!(XrayBumpNaming::is_conventional_bump(BUMP) && !XrayBumpNaming::is_conventional_bump(BASE));
  assert!(
    XrayBumpNaming::carries_marker(COMPANION) && XrayBumpNaming::carries_marker("ston\\my_bumpmap"),
    "the renderer tests a substring, so the marker can sit anywhere"
  );
}
