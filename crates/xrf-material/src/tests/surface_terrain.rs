//! The four details and mask a terrain class lays over its base, as the deferred renderers compile `B_BmmD`.

use xrf_shaders::fixtures::ShaderBlenderFixture;
use xrf_vfs::{XrayMountId, XrayProbe, XrayVfs};

use crate::fixtures::FixtureTree;
use crate::tests::material_probe::probe_over;
use crate::{
  XraySurfaceDescriptor, XraySurfaceResolver, XraySurfaceTerrain, XraySurfaceTerrainLayer, XrayTextureScope,
};

/// Describes `shader` in a tree defining `blender`, dressed as a level dresses its terrain: the base, then its lightmap.
fn describe_terrain(case: &str, blender: ShaderBlenderFixture, shader: &str) -> XraySurfaceDescriptor {
  let tree: FixtureTree = FixtureTree::new(case).with_shader_library(&[blender]);
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe<'_> = probe_over(&vfs, id);
  let textures: Vec<String> = vec![
    String::from("terrain\\terrain_zaton"),
    String::from("terrain\\terrain_zaton_lm"),
  ];

  XraySurfaceResolver::open(&probe, XrayTextureScope::shared()).describe(shader, &textures)
}

fn layer(reference: &str) -> XraySurfaceTerrainLayer {
  XraySurfaceTerrainLayer {
    reference: reference.to_owned(),
    bump: format!("{reference}_bump"),
  }
}

#[test]
fn a_terrain_lays_its_four_details_by_the_mask_beside_its_base() {
  let blender: ShaderBlenderFixture =
    ShaderBlenderFixture::level_detailed("levels\\zaton_earth", "detail\\detail_grnd_earth");

  assert_eq!(
    describe_terrain("terrain_layers", blender, "levels\\zaton_earth").terrain,
    Some(XraySurfaceTerrain {
      mask: String::from("terrain\\terrain_zaton_mask"),
      layers: [
        layer("detail\\detail_grnd_grass"),
        layer("detail\\detail_grnd_asphalt"),
        layer("detail\\detail_grnd_earth"),
        layer("detail\\detail_grnd_yantar"),
      ],
    })
  );
}

#[test]
fn a_terrain_naming_no_detail_of_its_four_and_any_other_class_lay_none() {
  let unnamed: ShaderBlenderFixture =
    ShaderBlenderFixture::level_detailed("levels\\old_earth", "detail\\detail_grnd_earth").without_property("R2-A");

  assert_eq!(
    describe_terrain("terrain_unnamed", unnamed, "levels\\old_earth").terrain,
    None
  );
  assert_eq!(
    describe_terrain(
      "terrain_model",
      ShaderBlenderFixture::model("models\\model"),
      "models\\model"
    )
    .terrain,
    None
  );
}
