//! The lighting model a descriptor sets, `m_material`.

use xrf_thm::{ThmMaterial, ThmTextureType};

use crate::XrayMaterialDescriptor;
use crate::fixtures::{FixtureTree, ThmFixture};
use crate::tests::material_probe::{BASE, describe};

#[test]
fn a_descriptor_sets_its_class_plus_its_weight() {
  let descriptor: XrayMaterialDescriptor = describe(
    &FixtureTree::new("material_weighted")
      .with_texture(BASE)
      .with_descriptor(BASE, &ThmFixture::image().with_material(ThmMaterial::PhongMetal, 0.5)),
  );

  assert_eq!(descriptor.material, 2.5);
}

#[test]
fn a_descriptor_without_a_material_chunk_shades_as_the_sdk_default() {
  let mut fixture: ThmFixture = ThmFixture::image();

  fixture.file.material = None;

  let descriptor: XrayMaterialDescriptor = describe(
    &FixtureTree::new("material_absent")
      .with_texture(BASE)
      .with_descriptor(BASE, &fixture),
  );

  assert_eq!(descriptor.material, XrayMaterialDescriptor::DEFAULT_MATERIAL);
}

#[test]
fn a_texture_the_engine_reads_no_descriptor_for_keeps_its_own_default() {
  let absent: XrayMaterialDescriptor = describe(&FixtureTree::new("material_undeclared").with_texture(BASE));
  let skipped: XrayMaterialDescriptor = describe(
    &FixtureTree::new("material_disqualified")
      .with_texture(BASE)
      .with_descriptor(
        BASE,
        &ThmFixture::image()
          .with_texture_type(ThmTextureType::CubeMap)
          .with_material(ThmMaterial::MetalOrenNayar, 0.25),
      ),
  );

  assert_eq!(absent.material, XrayMaterialDescriptor::DEFAULT_MATERIAL);
  assert_eq!(skipped.material, XrayMaterialDescriptor::DEFAULT_MATERIAL);
}
