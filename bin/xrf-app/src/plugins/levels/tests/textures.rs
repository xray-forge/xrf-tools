//! What a level's texture references come to, beside the level and in the shared tree.

use xrf_level::{LevelFile, LevelHeaderChunk, LevelShaderEntry, LevelShadersChunk};
use xrf_material::fixtures::FixtureTree;
use xrf_vfs::{XrayLookupScope, XrayMountId, XrayProbe, XrayVfs};

use crate::plugins::levels::state::LevelTextureReference;
use crate::plugins::levels::textures::resolve_textures;

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

fn new_resolved(tree: &FixtureTree, level: &LevelFile, directory: Option<&str>) -> Vec<LevelTextureReference> {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe = vfs.probe().with_step("tree", XrayLookupScope::only([id]));

  resolve_textures(level, &probe, directory)
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

#[test]
fn a_level_carrying_no_shader_table_names_no_textures() {
  let tree: FixtureTree = FixtureTree::new("level_textures_no_table").with_texture("prop\\prop_fence");
  let mut bare: LevelFile = new_level(&[]);

  bare.shaders = None;

  assert!(new_resolved(&tree, &bare, Some(DIRECTORY)).is_empty());
}
