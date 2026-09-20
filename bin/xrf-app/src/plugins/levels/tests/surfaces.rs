//! What a level's shader table comes to, which is what decides whether its foliage is cut out or drawn as a card,
//! and what its ground is detailed with.

use xrf_level::{LevelFile, LevelHeaderChunk, LevelShaderEntry, LevelShadersChunk};
use xrf_material::fixtures::{FixtureTree, ThmFixture};
use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDetail, XraySurfaceDraw};
use xrf_shaders::fixtures::ShaderBlenderFixture;
use xrf_thm::ThmTextureFlag;
use xrf_vfs::{XrayLookupScope, XrayMountId, XrayProbe, XrayVfs};

use crate::plugins::levels::surfaces::resolve_surfaces;

const CUT_OUT: XraySurfaceDraw = XraySurfaceDraw::AlphaTested {
  reference: XraySurfaceDraw::DEFERRED_ALPHA_REFERENCE,
};

/// A level whose table holds exactly these raw entries.
fn new_level(entries: &[&str]) -> LevelFile {
  LevelFile {
    header: LevelHeaderChunk {
      xrlc_quality: 1,
      xrlc_version: 14,
    },
    lights: None,
    portals: None,
    sectors: None,
    shaders: Some(LevelShadersChunk {
      entries: entries.iter().map(|raw| LevelShaderEntry::parse(raw)).collect(),
    }),
  }
}

fn new_resolved(tree: &FixtureTree, level: &LevelFile) -> Vec<XraySurfaceDescriptor> {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe = vfs.probe().with_step("tree", XrayLookupScope::only([id]));

  resolve_surfaces(level, &probe)
}

#[test]
fn a_level_takes_its_cut_out_and_its_opaque_surfaces_from_the_library() {
  let tree: FixtureTree = FixtureTree::new("level_surfaces_classes").with_shader_library(&[
    ShaderBlenderFixture::tree("levels\\tree").with_alpha_channel(true),
    ShaderBlenderFixture::level_aref("levels\\aref"),
    ShaderBlenderFixture::of(xrf_shaders::ShaderBlenderClass::DEFAULT, "levels\\solid"),
  ]);

  let surfaces: Vec<XraySurfaceDescriptor> = new_resolved(
    &tree,
    &new_level(&[
      "levels\\tree/veg\\veg_leaves",
      "levels\\aref/veg\\veg_grass",
      "levels\\solid/prop\\prop_wall",
    ]),
  );

  // The two classes a marsh is mostly made of, and the reason its reeds were drawn as solid cards.
  assert_eq!(surfaces[0].draw, CUT_OUT);
  assert_eq!(surfaces[1].draw, CUT_OUT);
  assert_eq!(surfaces[2].draw, XraySurfaceDraw::Opaque);
}

// A shader id indexes the answers, so a row the renderer resolves nothing for still occupies its own place rather
// than shifting every row after it onto the wrong surface.
#[test]
fn every_row_of_the_table_is_answered_for_in_place() {
  let tree: FixtureTree =
    FixtureTree::new("level_surfaces_shared").with_shader_library(&[ShaderBlenderFixture::level_aref("levels\\aref")]);

  let surfaces: Vec<XraySurfaceDescriptor> = new_resolved(
    &tree,
    &new_level(&[
      "",
      "levels\\aref/veg\\veg_grass",
      "no_delimiter",
      "levels\\aref/veg\\veg_reed",
    ]),
  );

  assert_eq!(surfaces.len(), 4, "one answer per table row, not per shader name");
  assert_eq!(surfaces[0].declaration, XraySurfaceDeclaration::Undeclared);
  assert_eq!(surfaces[1].draw, CUT_OUT);
  assert_eq!(surfaces[2].declaration, XraySurfaceDeclaration::Undeclared);
  assert_eq!(surfaces[3].draw, CUT_OUT);
}

#[test]
fn a_shader_the_library_does_not_define_is_named_as_undefined_rather_than_omitted() {
  // Omitting it would draw the surface opaque with nothing to say why, which is the state this whole pass exists to
  // tell apart from a blender that really is opaque.
  let tree: FixtureTree = FixtureTree::new("level_surfaces_undefined")
    .with_shader_library(&[ShaderBlenderFixture::level_aref("levels\\aref")]);

  let surfaces: Vec<XraySurfaceDescriptor> = new_resolved(&tree, &new_level(&["levels\\missing/prop\\wall"]));

  assert_eq!(surfaces[0].declaration, XraySurfaceDeclaration::Undefined);
  assert_eq!(surfaces[0].draw, XraySurfaceDraw::Opaque);
}

#[test]
fn a_tree_with_no_library_still_answers_for_every_shader() {
  let tree: FixtureTree = FixtureTree::new("level_surfaces_no_library");

  let surfaces: Vec<XraySurfaceDescriptor> = new_resolved(&tree, &new_level(&["levels\\aref/veg\\veg_grass"]));

  assert_eq!(
    surfaces[0].declaration,
    XraySurfaceDeclaration::NoLibrary,
    "a viewer can say no library was found rather than showing every surface as authored opaque"
  );
}

#[test]
fn a_level_carrying_no_shader_table_names_no_surfaces() {
  let tree: FixtureTree = FixtureTree::new("level_surfaces_no_table")
    .with_shader_library(&[ShaderBlenderFixture::level_aref("levels\\aref")]);
  let mut bare: LevelFile = new_level(&[]);

  bare.shaders = None;

  assert!(new_resolved(&tree, &bare).is_empty());
}

// The ground of an outdoor level: a base texture covering the whole terrain, and the tiled texture the engine
// multiplies over it, which is what makes it read as ground rather than as an aerial photograph.
#[test]
fn a_detailed_class_takes_the_texture_its_blender_names_and_the_tiling_its_base_texture_sets() {
  let tree: FixtureTree = FixtureTree::new("level_surfaces_detail_declared")
    .with_shader_library(&[ShaderBlenderFixture::level_detailed(
      "levels\\marsh_earth",
      "detail\\detail_grnd_earth",
    )])
    .with_descriptor(
      "terrain\\terrain_marsh",
      &ThmFixture::image().with_detail("detail\\detail_grnd_yantar", 150.0, &[ThmTextureFlag::DiffuseDetail]),
    );

  let surfaces: Vec<XraySurfaceDescriptor> = new_resolved(
    &tree,
    &new_level(&["levels\\marsh_earth/terrain\\terrain_marsh,terrain\\terrain_marsh_lm"]),
  );

  assert_eq!(
    surfaces[0].detail,
    Some(XraySurfaceDetail {
      // The blender's own name wins over the one the descriptor associates, which is what `CBlender_BmmD` binds.
      reference: String::from("detail\\detail_grnd_earth"),
      // The tiling never comes from anywhere but the descriptor, whichever texture is bound.
      scale: 150.0,
    })
  );
}

#[test]
fn a_class_with_no_detail_of_its_own_takes_the_one_its_base_texture_associates() {
  let tree: FixtureTree = FixtureTree::new("level_surfaces_detail_associated")
    .with_shader_library(&[ShaderBlenderFixture::of(
      xrf_shaders::ShaderBlenderClass::DEFAULT,
      "levels\\wall",
    )])
    .with_descriptor(
      "crete\\crete_beton_7",
      &ThmFixture::image().with_detail("detail\\detail_beton_det4", 8.0, &[ThmTextureFlag::DiffuseDetail]),
    );

  let surfaces: Vec<XraySurfaceDescriptor> =
    new_resolved(&tree, &new_level(&["levels\\wall/crete\\crete_beton_7,lmap#1_1"]));

  assert_eq!(
    surfaces[0].detail,
    Some(XraySurfaceDetail {
      reference: String::from("detail\\detail_beton_det4"),
      scale: 8.0,
    })
  );
}

// Two rows over one shader, detailed apart by the textures they dress with, which is the whole reason an answer is
// kept per row rather than per name.
#[test]
fn two_rows_naming_one_shader_are_detailed_by_their_own_base_textures() {
  let tree: FixtureTree = FixtureTree::new("level_surfaces_detail_per_row")
    .with_shader_library(&[ShaderBlenderFixture::of(
      xrf_shaders::ShaderBlenderClass::DEFAULT,
      "levels\\ground",
    )])
    .with_descriptor(
      "grnd\\grnd_dirt",
      &ThmFixture::image().with_detail("detail\\detail_dirt_det1", 8.0, &[ThmTextureFlag::DiffuseDetail]),
    )
    .with_descriptor(
      "grnd\\grnd_grass",
      &ThmFixture::image().with_detail("detail\\detail_grnd_grass", 4.0, &[ThmTextureFlag::DiffuseDetail]),
    );

  let surfaces: Vec<XraySurfaceDescriptor> = new_resolved(
    &tree,
    &new_level(&["levels\\ground/grnd\\grnd_dirt", "levels\\ground/grnd\\grnd_grass"]),
  );

  assert_eq!(
    surfaces[0].detail.as_ref().map(|it| it.reference.as_str()),
    Some("detail\\detail_dirt_det1")
  );
  assert_eq!(
    surfaces[1].detail.as_ref().map(|it| it.reference.as_str()),
    Some("detail\\detail_grnd_grass")
  );
}

#[test]
fn a_detail_association_with_neither_flag_set_details_nothing() {
  // The engine reads the flags before it binds anything, so authoring a name without one is dead authoring rather
  // than a detail texture the viewer should invent.
  let tree: FixtureTree = FixtureTree::new("level_surfaces_detail_unflagged")
    .with_shader_library(&[ShaderBlenderFixture::of(
      xrf_shaders::ShaderBlenderClass::DEFAULT,
      "levels\\wall",
    )])
    .with_descriptor(
      "crete\\crete_beton_7",
      &ThmFixture::image().with_detail("detail\\detail_beton_det4", 8.0, &[]),
    );

  let surfaces: Vec<XraySurfaceDescriptor> = new_resolved(&tree, &new_level(&["levels\\wall/crete\\crete_beton_7"]));

  assert_eq!(surfaces[0].detail, None);
}

#[test]
fn a_class_the_engine_never_details_takes_no_detail_from_its_base_texture() {
  // `B_MODEL` answers `canBeDetailed` with the base class's `FALSE`, so an association on its texture is not one the
  // renderer ever binds.
  let tree: FixtureTree = FixtureTree::new("level_surfaces_detail_undetailable")
    .with_shader_library(&[ShaderBlenderFixture::model("models\\model")])
    .with_descriptor(
      "prop\\prop_wall",
      &ThmFixture::image().with_detail("detail\\detail_beton_det4", 8.0, &[ThmTextureFlag::DiffuseDetail]),
    );

  let surfaces: Vec<XraySurfaceDescriptor> = new_resolved(&tree, &new_level(&["models\\model/prop\\prop_wall"]));

  assert_eq!(surfaces[0].detail, None);
}
