//! The detail association, live or dead, and the type gate in front of it.

use xrf_db::{ThmTextureParamChunk, ThmTextureTypeChunk};

use crate::fixtures::{ThmFixture, ThmFixtureTree};
use crate::tests::material_probe::{BASE, describe};
use crate::{XrayDetailUsage, XrayMaterialDescriptor};

#[test]
fn a_detail_with_the_bump_flag_is_a_bump_detail() {
  let descriptor: XrayMaterialDescriptor =
    describe(&ThmFixtureTree::new("detail_bump").with_texture(BASE).with_descriptor(
      BASE,
      &ThmFixture::image().with_detail("detail\\detail_grnd_grass", 4.0, ThmTextureParamChunk::FLAG_BUMP_DETAIL),
    ));

  let detail = descriptor.detail.expect("a named detail is reported");

  assert_eq!(detail.name, "detail\\detail_grnd_grass");
  assert_eq!(detail.scale, 4.0);
  assert_eq!(detail.usage, Some(XrayDetailUsage::Bump));
}

#[test]
fn a_detail_with_both_flags_is_diffuse_and_bump() {
  let descriptor: XrayMaterialDescriptor =
    describe(&ThmFixtureTree::new("detail_both").with_texture(BASE).with_descriptor(
      BASE,
      &ThmFixture::image().with_detail(
        "detail\\detail_grnd_grass",
        6.0,
        ThmTextureParamChunk::FLAG_DIFFUSE_DETAIL | ThmTextureParamChunk::FLAG_BUMP_DETAIL,
      ),
    ));

  assert_eq!(
    descriptor.detail.expect("reported").usage,
    Some(XrayDetailUsage::DiffuseAndBump)
  );
}

#[test]
fn a_detail_name_without_a_flag_is_reported_as_not_applied() {
  let descriptor: XrayMaterialDescriptor =
    describe(&ThmFixtureTree::new("detail_dead").with_texture(BASE).with_descriptor(
      BASE,
      &ThmFixture::image().with_detail("detail\\detail_grnd_grass", 4.0, 0),
    ));

  let detail = descriptor.detail.expect("dead authoring is still reported");

  assert_eq!(detail.usage, None);
}

#[test]
fn a_detail_is_not_read_from_a_disqualified_descriptor() {
  let descriptor: XrayMaterialDescriptor = describe(
    &ThmFixtureTree::new("detail_disqualified")
      .with_texture(BASE)
      .with_descriptor(
        BASE,
        &ThmFixture::image()
          .with_texture_type(ThmTextureTypeChunk::CUBE_MAP)
          .with_detail(
            "detail\\detail_grnd_grass",
            4.0,
            ThmTextureParamChunk::FLAG_DIFFUSE_DETAIL,
          ),
      ),
  );

  assert_eq!(descriptor.detail, None);
}
