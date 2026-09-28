//! The bump pair and lighting model a surface takes from its base texture's descriptor, by the class it compiles as.

use xrf_shaders::ShaderBlenderClass;
use xrf_shaders::fixtures::ShaderBlenderFixture;
use xrf_thm::ThmMaterial;
use xrf_vfs::{XrayMountId, XrayProbe, XrayVfs};

use crate::fixtures::FixtureTree;
use crate::tests::material_probe::{BASE, BUMP, COMPANION, located_path, probe_over, used_bump};
use crate::{XrayMaterialDescriptor, XraySurfaceDescriptor, XraySurfaceResolver};

/// A tree whose library defines `blender` and whose base texture declares a located pair, as `PhongMetal` at a half.
fn bumped_tree(case: &str, blender: ShaderBlenderFixture) -> FixtureTree {
  FixtureTree::new(case)
    .with_shader_library(&[blender])
    .with_texture(BASE)
    .with_texture(BUMP)
    .with_texture(COMPANION)
    .with_descriptor(BASE, &used_bump().with_material(ThmMaterial::PhongMetal, 0.5))
}

/// Describes `shader` dressed with the base texture alone.
fn describe(tree: &FixtureTree, shader: &str) -> XraySurfaceDescriptor {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe<'_> = probe_over(&vfs, id);

  XraySurfaceResolver::open(&probe).describe(shader, &[BASE.to_owned()])
}

#[test]
fn a_lightmapped_surface_binds_its_base_texture_pair_and_shades_with_its_model() {
  let tree: FixtureTree = bumped_tree(
    "surface_bump_default",
    ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "default"),
  );
  let descriptor: XraySurfaceDescriptor = describe(&tree, "default");
  let bump = descriptor.bump.expect("the class binds the pair its base declares");

  assert_eq!(bump.bump.reference, BUMP);
  assert_eq!(bump.companion.reference, COMPANION);
  assert_eq!(
    located_path(&bump.bump.resolution),
    Some("textures\\act\\act_stalker_bump.dds")
  );
  assert_eq!(descriptor.material, 2.5);
}

#[test]
fn every_class_compiling_through_uber_deffer_binds_the_pair() {
  for (case, name, blender) in [
    (
      "surface_bump_model",
      "models\\model",
      ShaderBlenderFixture::model("models\\model"),
    ),
    (
      "surface_bump_tree",
      "trees\\trees",
      ShaderBlenderFixture::tree("trees\\trees"),
    ),
    (
      "surface_bump_aref",
      "def_shaders\\def_aref",
      ShaderBlenderFixture::level_aref("def_shaders\\def_aref"),
    ),
  ] {
    let descriptor: XraySurfaceDescriptor = describe(&bumped_tree(case, blender), name);

    assert!(descriptor.bump.is_some(), "{case} binds the pair");
  }
}

// `uber_deffer` would bind grass's pair as it binds any other's, but no tree ships a `deffer_detail_*_bump` program to
// draw it with: grass draws flat, the only way the engine can.
#[test]
fn grass_binds_no_pair_whatever_its_base_declares() {
  let tree: FixtureTree = bumped_tree("surface_bump_detail", ShaderBlenderFixture::detail("details\\blend"));
  let descriptor: XraySurfaceDescriptor = describe(&tree, "details\\blend");

  assert_eq!(descriptor.bump, None);
  // The lighting model is the base texture's all the same: `L_material` reads the texture, not the class.
  assert_eq!(descriptor.material, 2.5);
}

#[test]
fn a_base_texture_without_a_descriptor_draws_flat_with_the_engine_default() {
  let tree: FixtureTree = FixtureTree::new("surface_bump_undeclared")
    .with_shader_library(&[ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "default")])
    .with_texture(BASE);
  let descriptor: XraySurfaceDescriptor = describe(&tree, "default");

  assert_eq!(descriptor.bump, None);
  assert_eq!(descriptor.material, XrayMaterialDescriptor::DEFAULT_MATERIAL);
}
