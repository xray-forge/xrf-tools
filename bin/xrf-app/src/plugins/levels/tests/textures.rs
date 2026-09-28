//! What a level's texture references come to, beside the level and in the shared tree.

use std::collections::BTreeSet;
use xrf_level::{LevelFile, LevelHeaderChunk, LevelShaderEntry, LevelShadersChunk};

use xrf_material::fixtures::{FixtureTree, ThmFixture};
use xrf_material::{XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope};
use xrf_shaders::ShaderBlenderClass;
use xrf_shaders::fixtures::ShaderBlenderFixture;
use xrf_thm::{ThmBumpMode, ThmTextureFlag};
use xrf_vfs::{XrayLogicalPath, XrayLookupScope, XrayMountId, XrayProbe, XrayResolution, XrayVfs};

use crate::plugins::levels::state::LevelTextureReference;
use crate::plugins::levels::surfaces::resolve_surfaces;
use crate::plugins::levels::textures::{resolve_level_textures, resolve_surface_textures};

const LEVEL: &str = "k00_marsh";
const DIRECTORY: &str = "levels\\k00_marsh";

/// A level whose one table entry names a base texture and the two lightmaps after it.
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

/// The level's own directory as the engine mounts it, or the shared tree alone for a level with none.
fn new_scope(directory: Option<&str>) -> XrayTextureScope {
  directory.map_or_else(XrayTextureScope::shared, |directory| {
    XrayTextureScope::of_level(XrayLogicalPath::new(directory).expect("a level directory"))
  })
}

/// The level's surfaces and the textures they bind, as an open resolves both.
fn new_opened(
  tree: &FixtureTree,
  level: &LevelFile,
  directory: Option<&str>,
) -> (Vec<XraySurfaceDescriptor>, Vec<LevelTextureReference>) {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe = vfs.probe().with_step("tree", XrayLookupScope::only([id]));
  let scope: XrayTextureScope = new_scope(directory);
  let surfaces: Vec<XraySurfaceDescriptor> = resolve_surfaces(level, &probe, &scope);
  let references: Vec<LevelTextureReference> = resolve_level_textures(level, &surfaces, &probe, &scope);

  (surfaces, references)
}

fn new_resolved(tree: &FixtureTree, level: &LevelFile, directory: Option<&str>) -> Vec<LevelTextureReference> {
  new_opened(tree, level, directory).1
}

fn get_path_of<'a>(references: &'a [LevelTextureReference], reference: &str) -> Option<&'a str> {
  references
    .iter()
    .find(|it| it.reference == reference)
    .and_then(|it| it.logical_path.as_deref())
}

#[test]
fn finds_a_lightmap_beside_the_level_where_the_shared_tree_holds_none() {
  let tree: FixtureTree = FixtureTree::new("level_textures_lightmap")
    .with_texture("prop\\prop_fence")
    .with_level_texture(LEVEL, "lmap#1_1");

  let references: Vec<LevelTextureReference> = new_resolved(
    &tree,
    &new_level(&["def_shaders\\lm/prop\\prop_fence,lmap#1_1"]),
    Some(DIRECTORY),
  );

  assert_eq!(
    get_path_of(&references, "lmap#1_1"),
    Some("levels\\k00_marsh\\lmap#1_1.dds"),
    "a lightmap exists only beside its level"
  );
  assert_eq!(
    get_path_of(&references, "prop\\prop_fence"),
    Some("textures\\prop\\prop_fence.dds")
  );
}

#[test]
fn a_level_copy_wins_over_the_shared_tree() {
  // `CTexture::Load` searches `$level$` first, so a level shipping its own copy of a shared name draws that copy.
  let tree: FixtureTree = FixtureTree::new("level_textures_override")
    .with_texture("prop\\prop_fence")
    .with_level_texture(LEVEL, "prop\\prop_fence");

  let references: Vec<LevelTextureReference> = new_resolved(
    &tree,
    &new_level(&["def_shaders\\lm/prop\\prop_fence"]),
    Some(DIRECTORY),
  );

  assert_eq!(
    get_path_of(&references, "prop\\prop_fence"),
    Some("levels\\k00_marsh\\prop\\prop_fence.dds")
  );
}

#[test]
fn falls_back_to_the_shared_tree_and_reports_what_neither_holds() {
  let tree: FixtureTree = FixtureTree::new("level_textures_missing").with_texture("prop\\prop_fence");

  let references: Vec<LevelTextureReference> = new_resolved(
    &tree,
    &new_level(&["def_shaders\\lm/prop\\prop_fence,lmap#1_1"]),
    Some(DIRECTORY),
  );

  assert_eq!(
    get_path_of(&references, "prop\\prop_fence"),
    Some("textures\\prop\\prop_fence.dds")
  );
  assert_eq!(
    get_path_of(&references, "lmap#1_1"),
    None,
    "a reference neither place holds resolves to nothing rather than to the wrong file"
  );
}

#[test]
fn a_level_with_no_engine_identity_still_resolves_the_shared_tree() {
  let tree: FixtureTree = FixtureTree::new("level_textures_no_directory")
    .with_texture("prop\\prop_fence")
    .with_level_texture(LEVEL, "lmap#1_1");

  let references: Vec<LevelTextureReference> =
    new_resolved(&tree, &new_level(&["def_shaders\\lm/prop\\prop_fence,lmap#1_1"]), None);

  assert_eq!(
    get_path_of(&references, "prop\\prop_fence"),
    Some("textures\\prop\\prop_fence.dds")
  );
  assert_eq!(get_path_of(&references, "lmap#1_1"), None);
}

#[test]
fn names_every_reference_once_however_many_entries_share_it() {
  let tree: FixtureTree = FixtureTree::new("level_textures_shared").with_texture("prop\\prop_fence");

  let references: Vec<LevelTextureReference> = new_resolved(
    &tree,
    &new_level(&[
      "def_shaders\\lm/prop\\prop_fence",
      "def_shaders\\aref/prop\\prop_fence",
      "",
    ]),
    Some(DIRECTORY),
  );

  assert_eq!(references.len(), 1);
}

// The detail texture is named nowhere in the level: the shader table lists the base and the lightmaps, and the
// detail comes off the base texture's descriptor. Resolving only what the table names leaves the ground undetailed.
#[test]
fn resolves_the_detail_texture_a_surface_binds_though_the_table_never_names_it() {
  let tree: FixtureTree = FixtureTree::new("level_textures_detail")
    .with_shader_library(&[ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "levels\\ground")])
    .with_descriptor(
      "grnd\\grnd_dirt",
      &ThmFixture::image().with_detail("detail\\detail_dirt_det1", 8.0, &[ThmTextureFlag::DiffuseDetail]),
    )
    .with_texture("grnd\\grnd_dirt")
    .with_texture("detail\\detail_dirt_det1");

  let references: Vec<LevelTextureReference> =
    new_resolved(&tree, &new_level(&["levels\\ground/grnd\\grnd_dirt"]), Some(DIRECTORY));

  assert_eq!(
    get_path_of(&references, "detail\\detail_dirt_det1"),
    Some("textures\\detail\\detail_dirt_det1.dds")
  );
}

// `effects_water.s` binds its normal map and foam by name, which no shader table entry spells.
#[test]
fn resolves_the_textures_a_surface_script_binds_by_sampler() {
  let tree: FixtureTree = FixtureTree::new("level_textures_script")
    .with_shader_library(&[])
    .with_shader_script(
      "effects\\water",
      r#"
local tex_nmap = "water\\water_normal"

function normal (shader, t_base, t_second, t_detail)
  shader:begin ("water_soft","water_soft") : blend (true,blend.srcalpha,blend.invsrcalpha)
  shader:sampler ("s_nmap")   :texture (tex_nmap)
  shader:sampler ("s_leaves") :texture ("water\\water_foam")
  shader:sampler ("s_env0")   :texture ("$user$sky0")
end
"#,
    )
    .with_texture("water\\water_normal")
    .with_texture("water\\water_foam");

  let references: Vec<LevelTextureReference> = new_resolved(
    &tree,
    &new_level(&["effects\\water/water\\water_water"]),
    Some(DIRECTORY),
  );

  assert_eq!(
    get_path_of(&references, "water\\water_normal"),
    Some("textures\\water\\water_normal.dds")
  );
  assert_eq!(
    get_path_of(&references, "water\\water_foam"),
    Some("textures\\water\\water_foam.dds")
  );
  assert!(references.iter().all(|it| !it.reference.starts_with('$')));
}

// A spawned model names only its base textures; the pair and the detail it shades with come off the base's
// descriptor, as a level surface's do, for a class binding both: a vertex-lit prop's.
#[test]
fn resolves_the_bump_pair_and_detail_a_models_surface_binds_beside_its_base() {
  let tree: FixtureTree = FixtureTree::new("level_textures_model")
    .with_shader_library(&[ShaderBlenderFixture::of(
      ShaderBlenderClass::VERT,
      "def_shaders\\def_vertex",
    )])
    .with_descriptor(
      "lights\\lights_lamp",
      &ThmFixture::image()
        .with_bump(ThmBumpMode::Use, "lights\\lights_lamp_bump")
        .with_detail("detail\\detail_metal", 4.0, &[ThmTextureFlag::DiffuseDetail]),
    )
    .with_texture("lights\\lights_lamp")
    .with_texture("lights\\lights_lamp_bump")
    .with_texture("lights\\lights_lamp_bump#")
    .with_texture("detail\\detail_metal");
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe = vfs.probe().with_step("tree", XrayLookupScope::only([id]));
  let surfaces: Vec<XraySurfaceDescriptor> = vec![
    XraySurfaceResolver::open(&probe, XrayTextureScope::shared())
      .describe("def_shaders\\def_vertex", &["lights\\lights_lamp".to_owned()]),
  ];

  let references: Vec<LevelTextureReference> = resolve_surface_textures(
    BTreeSet::from(["lights\\lights_lamp".to_owned()]),
    &surfaces,
    &probe,
    &XrayTextureScope::shared(),
  );

  assert_eq!(
    get_path_of(&references, "lights\\lights_lamp"),
    Some("textures\\lights\\lights_lamp.dds")
  );
  assert_eq!(
    get_path_of(&references, "lights\\lights_lamp_bump"),
    Some("textures\\lights\\lights_lamp_bump.dds")
  );
  assert_eq!(
    get_path_of(&references, "lights\\lights_lamp_bump#"),
    Some("textures\\lights\\lights_lamp_bump#.dds")
  );
  assert_eq!(
    get_path_of(&references, "detail\\detail_metal"),
    Some("textures\\detail\\detail_metal.dds")
  );
}

/// Where a surface's bump pair resolved, as its descriptor reported it.
fn get_bump_paths(surface: &XraySurfaceDescriptor) -> (Option<&str>, Option<&str>) {
  surface.bump.as_ref().map_or((None, None), |bump| {
    (
      get_located_path(&bump.bump.resolution),
      get_located_path(&bump.companion.resolution),
    )
  })
}

fn get_located_path(resolution: &XrayResolution) -> Option<&str> {
  resolution.get_asset().map(|asset| asset.get_logical_path().as_str())
}

/// A tree whose ground declares the bump pair `grnd\grnd_dirt_bump`, the engine's dummies beside it.
fn new_bumped_tree(name: &str) -> FixtureTree {
  FixtureTree::new(name)
    .with_shader_library(&[ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "levels\\ground")])
    .with_descriptor(
      "grnd\\grnd_dirt",
      &ThmFixture::image().with_bump(ThmBumpMode::Use, "grnd\\grnd_dirt_bump"),
    )
    .with_texture("grnd\\grnd_dirt")
    .with_texture("ed\\ed_dummy_bump")
    .with_texture("ed\\ed_dummy_bump#")
}

// `texture_load` searches `$level$` first for a bump too, once `$game_textures$` holds it.
#[test]
fn a_level_copy_of_a_bump_pair_wins_over_the_shared_tree() {
  let tree: FixtureTree = new_bumped_tree("level_textures_bump_override")
    .with_texture("grnd\\grnd_dirt_bump")
    .with_texture("grnd\\grnd_dirt_bump#")
    .with_level_texture(LEVEL, "grnd\\grnd_dirt_bump")
    .with_level_texture(LEVEL, "grnd\\grnd_dirt_bump#");

  let (surfaces, references): (Vec<XraySurfaceDescriptor>, Vec<LevelTextureReference>) =
    new_opened(&tree, &new_level(&["levels\\ground/grnd\\grnd_dirt"]), Some(DIRECTORY));

  assert_eq!(
    get_path_of(&references, "grnd\\grnd_dirt_bump"),
    Some("levels\\k00_marsh\\grnd\\grnd_dirt_bump.dds")
  );
  assert_eq!(
    get_path_of(&references, "grnd\\grnd_dirt_bump#"),
    Some("levels\\k00_marsh\\grnd\\grnd_dirt_bump#.dds")
  );
  assert_eq!(
    get_bump_paths(&surfaces[0]),
    (
      Some("levels\\k00_marsh\\grnd\\grnd_dirt_bump.dds"),
      Some("levels\\k00_marsh\\grnd\\grnd_dirt_bump#.dds")
    ),
    "the surface reports the file the level loads, not the shared one"
  );
}

// `LoadTHM` reads `$level$` after `$game_textures$`, so a level's own descriptor replaces the shared one's spec.
#[test]
fn a_level_descriptor_dresses_its_ground_over_the_shared_one() {
  let tree: FixtureTree = FixtureTree::new("level_textures_descriptor_override")
    .with_shader_library(&[ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "levels\\ground")])
    .with_descriptor("grnd\\grnd_dirt", &ThmFixture::image())
    .with_level_descriptor(
      LEVEL,
      "grnd\\grnd_dirt",
      &ThmFixture::image()
        .with_bump(ThmBumpMode::Use, "grnd\\grnd_dirt_bump")
        .with_detail("detail\\detail_dirt_det1", 8.0, &[ThmTextureFlag::DiffuseDetail]),
    )
    .with_texture("grnd\\grnd_dirt")
    .with_texture("grnd\\grnd_dirt_bump")
    .with_texture("grnd\\grnd_dirt_bump#")
    .with_texture("detail\\detail_dirt_det1");

  let (surfaces, references): (Vec<XraySurfaceDescriptor>, Vec<LevelTextureReference>) =
    new_opened(&tree, &new_level(&["levels\\ground/grnd\\grnd_dirt"]), Some(DIRECTORY));
  let (shared, _): (Vec<XraySurfaceDescriptor>, Vec<LevelTextureReference>) =
    new_opened(&tree, &new_level(&["levels\\ground/grnd\\grnd_dirt"]), None);

  assert_eq!(
    get_bump_paths(&surfaces[0]),
    (
      Some("textures\\grnd\\grnd_dirt_bump.dds"),
      Some("textures\\grnd\\grnd_dirt_bump#.dds")
    )
  );
  assert_eq!(
    get_path_of(&references, "detail\\detail_dirt_det1"),
    Some("textures\\detail\\detail_dirt_det1.dds")
  );
  assert_eq!(
    (shared[0].bump.as_ref(), shared[0].detail.as_ref()),
    (None, None),
    "a level with no engine identity reads the shared descriptor alone"
  );
}

// `texture_load` asks `$game_textures$` alone whether a `_bump` name exists, so a copy only the level holds is never
// loaded: the dummy is.
#[test]
fn a_bump_pair_the_shared_tree_lacks_is_the_dummy_though_the_level_holds_it() {
  let tree: FixtureTree = new_bumped_tree("level_textures_bump_level_only")
    .with_level_texture(LEVEL, "grnd\\grnd_dirt_bump")
    .with_level_texture(LEVEL, "grnd\\grnd_dirt_bump#");

  let references: Vec<LevelTextureReference> =
    new_resolved(&tree, &new_level(&["levels\\ground/grnd\\grnd_dirt"]), Some(DIRECTORY));

  assert_eq!(
    get_path_of(&references, "grnd\\grnd_dirt_bump"),
    Some("textures\\ed\\ed_dummy_bump.dds")
  );
  assert_eq!(
    get_path_of(&references, "grnd\\grnd_dirt_bump#"),
    Some("textures\\ed\\ed_dummy_bump#.dds")
  );
}

// Without `_bump` in its name, `texture_load` looks for a bump as for any texture, beside the level first.
#[test]
fn a_bump_named_without_the_marker_is_found_beside_the_level_alone() {
  let tree: FixtureTree = FixtureTree::new("level_textures_bump_unmarked")
    .with_shader_library(&[ShaderBlenderFixture::of(ShaderBlenderClass::DEFAULT, "levels\\ground")])
    .with_descriptor(
      "grnd\\grnd_dirt",
      &ThmFixture::image().with_bump(ThmBumpMode::Use, "grnd\\grnd_dirt_nm"),
    )
    .with_texture("grnd\\grnd_dirt")
    .with_texture("ed\\ed_not_existing_texture")
    .with_level_texture(LEVEL, "grnd\\grnd_dirt_nm")
    .with_level_texture(LEVEL, "grnd\\grnd_dirt_nm#");

  let references: Vec<LevelTextureReference> =
    new_resolved(&tree, &new_level(&["levels\\ground/grnd\\grnd_dirt"]), Some(DIRECTORY));

  assert_eq!(
    get_path_of(&references, "grnd\\grnd_dirt_nm"),
    Some("levels\\k00_marsh\\grnd\\grnd_dirt_nm.dds")
  );
  assert_eq!(
    get_path_of(&references, "grnd\\grnd_dirt_nm#"),
    Some("levels\\k00_marsh\\grnd\\grnd_dirt_nm#.dds")
  );
}

#[test]
fn a_level_carrying_no_shader_table_names_no_textures() {
  let tree: FixtureTree = FixtureTree::new("level_textures_no_table").with_texture("prop\\prop_fence");
  let mut bare: LevelFile = new_level(&[]);

  bare.shaders = None;

  assert!(new_resolved(&tree, &bare, Some(DIRECTORY)).is_empty());
}
