//! What a level's shader table comes to, which is what decides whether its foliage is cut out or drawn as a card.

use std::collections::HashMap;

use xrf_level::{LevelFile, LevelHeaderChunk, LevelShaderEntry, LevelShadersChunk};
use xrf_material::fixtures::FixtureTree;
use xrf_material::{XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDraw};
use xrf_shaders::fixtures::ShaderBlenderFixture;
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

fn new_resolved(tree: &FixtureTree, level: &LevelFile) -> HashMap<String, XraySurfaceDescriptor> {
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

  let surfaces: HashMap<String, XraySurfaceDescriptor> = new_resolved(
    &tree,
    &new_level(&[
      "levels\\tree/veg\\veg_leaves",
      "levels\\aref/veg\\veg_grass",
      "levels\\solid/prop\\prop_wall",
    ]),
  );

  // The two classes a marsh is mostly made of, and the reason its reeds were drawn as solid cards.
  assert_eq!(surfaces["levels\\tree"].draw, CUT_OUT);
  assert_eq!(surfaces["levels\\aref"].draw, CUT_OUT);
  assert_eq!(surfaces["levels\\solid"].draw, XraySurfaceDraw::Opaque);
}

#[test]
fn every_shader_the_table_names_is_described_once() {
  let tree: FixtureTree =
    FixtureTree::new("level_surfaces_shared").with_shader_library(&[ShaderBlenderFixture::level_aref("levels\\aref")]);

  let surfaces: HashMap<String, XraySurfaceDescriptor> = new_resolved(
    &tree,
    &new_level(&[
      "levels\\aref/veg\\veg_grass",
      "levels\\aref/veg\\veg_reed",
      "",
      "no_delimiter",
    ]),
  );

  assert_eq!(surfaces.len(), 1, "one entry per shader name, not per table row");
  assert!(surfaces.contains_key("levels\\aref"));
}

#[test]
fn a_shader_the_library_does_not_define_is_named_as_undefined_rather_than_omitted() {
  // Omitting it would draw the surface opaque with nothing to say why, which is the state this whole pass exists to
  // tell apart from a blender that really is opaque.
  let tree: FixtureTree = FixtureTree::new("level_surfaces_undefined")
    .with_shader_library(&[ShaderBlenderFixture::level_aref("levels\\aref")]);

  let surfaces: HashMap<String, XraySurfaceDescriptor> =
    new_resolved(&tree, &new_level(&["levels\\missing/prop\\wall"]));

  assert_eq!(
    surfaces["levels\\missing"].declaration,
    XraySurfaceDeclaration::Undefined
  );
  assert_eq!(surfaces["levels\\missing"].draw, XraySurfaceDraw::Opaque);
}

#[test]
fn a_tree_with_no_library_still_answers_for_every_shader() {
  let tree: FixtureTree = FixtureTree::new("level_surfaces_no_library");

  let surfaces: HashMap<String, XraySurfaceDescriptor> =
    new_resolved(&tree, &new_level(&["levels\\aref/veg\\veg_grass"]));

  assert_eq!(
    surfaces["levels\\aref"].declaration,
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
