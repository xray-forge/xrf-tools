//! The four details and mask a terrain class lays over its base, as the deferred renderers compile `B_BmmD`.

use xrf_shaders::fixtures::ShaderBlenderFixture;

use crate::tests::surface::surface_search::{describe_dressed, library};
use crate::{XraySurfaceDescriptor, XraySurfaceTerrain, XraySurfaceTerrainLayer};

/// Describes `shader` in a tree defining `blender`, dressed as a level dresses its terrain: the base, then its lightmap.
fn describe_terrain(case: &str, blender: ShaderBlenderFixture, shader: &str) -> XraySurfaceDescriptor {
  describe_dressed(
    &library(case, &[blender]),
    shader,
    &["terrain\\terrain_zaton", "terrain\\terrain_zaton_lm"],
  )
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
