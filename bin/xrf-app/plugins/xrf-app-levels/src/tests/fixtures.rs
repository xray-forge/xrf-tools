//! What several test modules build the same way: a level of shader entries, a search over a fixture tree, an empty
//! environment catalog, and a spawned object.

use xrf_engine_target::XrayEngine;
use xrf_environment::{EnvironmentCatalog, WeatherGraphs};
use xrf_level::{LevelFile, LevelHeaderChunk, LevelShaderEntry, LevelShadersChunk};
use xrf_material::fixtures::FixtureTree;
use xrf_math::Vector3d;
use xrf_spawn::{AlifeObject, AlifeObjectAbstract, AlifeObjectInherited, ClsId};
use xrf_vfs::{XrayLookupScope, XrayMountId, XraySearch, XrayVfs};

/// A level whose shader table holds exactly these raw entries.
pub fn new_level(entries: &[&str]) -> LevelFile {
  LevelFile {
    header: LevelHeaderChunk {
      xrlc_quality: 1,
      xrlc_version: 14,
    },
    glows: None,
    lights: None,
    portals: None,
    sectors: None,
    shaders: Some(LevelShadersChunk {
      entries: entries.iter().map(|raw| LevelShaderEntry::parse(raw)).collect(),
    }),
  }
}

/// A search over `tree` alone, mounted as the game's root.
pub fn mount_search(tree: &FixtureTree) -> XraySearch {
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");

  XraySearch::new(vfs).with_step("tree", XrayLookupScope::only([id]))
}

/// A catalog of nothing, the vanilla engine's, for a test to fill what it reads.
pub fn new_catalog() -> EnvironmentCatalog {
  EnvironmentCatalog {
    ambient_effects: Vec::new(),
    ambients: Vec::new(),
    configs: Vec::new(),
    cycles: Vec::new(),
    effects: Vec::new(),
    engine: XrayEngine::Vanilla,
    findings: Vec::new(),
    graphs: WeatherGraphs::default(),
    level_ambients: Vec::new(),
    sound_channels: Vec::new(),
    sun_table: None,
    suns: Vec::new(),
    thunderbolt_collections: Vec::new(),
    thunderbolt_settings: None,
    thunderbolts: Vec::new(),
  }
}

/// An object's abstract part with `story_id`, on no vertex.
pub fn new_abstract(story_id: u32) -> AlifeObjectAbstract {
  AlifeObjectAbstract {
    game_vertex_id: 0,
    distance: 0.0,
    direct_control: 1,
    level_vertex_id: 0,
    flags: 0,
    custom_data: String::new(),
    story_id,
    spawn_story_id: AlifeObjectAbstract::INVALID_STORY_ID,
  }
}

/// A spawned object named `name` of `section`, at `1, 2, 3` facing nowhere in particular.
pub fn new_object(name: &str, section: &str, clsid: ClsId, inherited: AlifeObjectInherited) -> AlifeObject {
  AlifeObject {
    id: 0,
    net_action: 1,
    section: String::from(section),
    clsid,
    name: String::from(name),
    script_game_id: 0,
    script_rp: 0,
    position: Vector3d::new(1.0, 2.0, 3.0),
    direction: Vector3d::new(0.0, 0.0, 0.0),
    respawn_time: 0,
    parent_id: u16::MAX,
    phantom_id: u16::MAX,
    script_flags: 0,
    version: 128,
    game_type: 0,
    script_version: 12,
    client_data: Vec::new(),
    spawn_id: 0,
    inherited,
    update_data: Vec::new(),
  }
}
