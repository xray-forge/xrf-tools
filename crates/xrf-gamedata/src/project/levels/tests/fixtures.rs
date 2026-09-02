use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use uuid::{Uuid, uuid};
use xrf_chunk::{ChunkReadWrite, ChunkWriter, XRayByteOrder};
use xrf_db::{
  GraphCrossTable, GraphHeader, GraphLevel, LevelAiHeader, LevelCformHeader, LevelHeaderChunk, LevelShaderEntry,
  LevelShadersChunk, SpawnGraphsChunk, Vector3d,
};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use crate::project::levels::level_engine_constants::{
  AI_CURRENT_VERSION, CFORM_CURRENT_VERSION, LEVEL_PRODUCTION_VERSION,
};
use crate::project::levels::verify_levels_result::GamedataLevelsVerificationResult;
use crate::{GamedataCheckResult, GamedataProject, GamedataProjectReadOptions, GamedataProjectVerifyOptions};

pub(crate) static NEXT_TEST_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) const GRAPH_GUID: Uuid = uuid!("11111111-1111-1111-1111-111111111111");
pub(crate) const ZATON_GUID: Uuid = uuid!("22222222-2222-2222-2222-222222222222");
pub(crate) const OTHER_GUID: Uuid = uuid!("33333333-3333-3333-3333-333333333333");
pub(crate) const AI_NODE_COUNT: u32 = 4096;

/// Declarative description of one level bundle written into a synthetic gamedata tree.
pub(crate) struct LevelBundleFixture {
  name: String,
  files: Vec<(String, Vec<u8>)>,
}

impl LevelBundleFixture {
  /// Bundle that passes every implemented rule.
  pub(crate) fn valid(name: &str) -> Self {
    Self {
      name: String::from(name),
      files: vec![
        (
          String::from("level"),
          level_file_bytes(LEVEL_PRODUCTION_VERSION, true, &["shader\\one/level\\ground"]),
        ),
        (
          String::from("level.ai"),
          ai_map_bytes(AI_CURRENT_VERSION, AI_NODE_COUNT, ZATON_GUID),
        ),
        (String::from("level.cform"), cform_bytes(CFORM_CURRENT_VERSION)),
        (String::from("level.game"), vec![1]),
        (String::from("level.geom"), vec![1]),
        (String::from("level.geomx"), vec![1]),
        (String::from("level.ltx"), level_ltx_bytes("map\\map_zaton")),
        (String::from("level.spawn"), vec![1]),
      ],
    }
  }

  pub(crate) fn without(mut self, file: &str) -> Self {
    self.files.retain(|(name, _)| name != file);
    self
  }

  pub(crate) fn with(mut self, file: &str, bytes: Vec<u8>) -> Self {
    self.files.retain(|(name, _)| name != file);
    self.files.push((String::from(file), bytes));
    self
  }

  pub(crate) fn write(&self, root: &Path) {
    let directory: PathBuf = root.join("levels").join(&self.name);

    fs::create_dir_all(&directory).unwrap();

    for (name, bytes) in &self.files {
      fs::write(directory.join(name), bytes).unwrap();
    }
  }
}

pub(crate) fn chunk_bytes(id: u32, payload: &[u8]) -> Vec<u8> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  writer.write_all(payload).unwrap();
  writer.flush_chunk_into_buffer::<XRayByteOrder>(id).unwrap()
}

pub(crate) fn level_file_bytes(version: u16, with_shaders: bool, references: &[&str]) -> Vec<u8> {
  let mut header_writer: ChunkWriter = ChunkWriter::new();

  LevelHeaderChunk {
    xrlc_version: version,
    xrlc_quality: 1,
  }
  .write::<XRayByteOrder>(&mut header_writer)
  .unwrap();

  let mut bytes: Vec<u8> = chunk_bytes(
    LevelHeaderChunk::CHUNK_ID,
    &header_writer.flush_raw_into_buffer().unwrap(),
  );

  if with_shaders {
    let mut shaders_writer: ChunkWriter = ChunkWriter::new();

    LevelShadersChunk {
      entries: references
        .iter()
        .map(|reference| LevelShaderEntry::parse(reference))
        .collect(),
    }
    .write::<XRayByteOrder>(&mut shaders_writer)
    .unwrap();

    bytes.extend(chunk_bytes(
      LevelShadersChunk::CHUNK_ID,
      &shaders_writer.flush_raw_into_buffer().unwrap(),
    ));
  }

  bytes
}

pub(crate) fn ai_map_bytes(version: u32, count: u32, guid: Uuid) -> Vec<u8> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelAiHeader {
    version,
    count,
    size: 0.7,
    size_y: 0.2,
    aabb_min: Vector3d::new(-1.0, -1.0, -1.0),
    aabb_max: Vector3d::new(1.0, 1.0, 1.0),
    guid,
  }
  .write::<XRayByteOrder>(&mut writer)
  .unwrap();

  writer.flush_raw_into_buffer().unwrap()
}

pub(crate) fn cform_bytes(version: u32) -> Vec<u8> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  LevelCformHeader {
    version,
    vertex_count: 10,
    face_count: 20,
    aabb_min: Vector3d::new(-1.0, -1.0, -1.0),
    aabb_max: Vector3d::new(1.0, 1.0, 1.0),
  }
  .write::<XRayByteOrder>(&mut writer)
  .unwrap();

  writer.flush_raw_into_buffer().unwrap()
}

pub(crate) fn level_ltx_bytes(texture: &str) -> Vec<u8> {
  format!("[level_map]\nbound_rect = -1.0, -1.0, 1.0, 1.0\ntexture = {texture}\n").into_bytes()
}

/// Build a spawn file containing only the game graph chunk, which is all the roster reads.
pub(crate) fn spawn_graph_bytes(levels: &[(&str, u8, Uuid)], cross_tables: Vec<GraphCrossTable>) -> Vec<u8> {
  let mut graphs_writer: ChunkWriter = ChunkWriter::new();

  let graphs: SpawnGraphsChunk = SpawnGraphsChunk {
    header: GraphHeader {
      version: 10,
      vertices_count: 0,
      edges_count: 0,
      points_count: 0,
      guid: GRAPH_GUID,
      levels_count: levels.len() as u8,
    },
    levels: levels
      .iter()
      .map(|(name, id, guid)| GraphLevel {
        name: String::from(*name),
        offset: Vector3d::new(0.0, 0.0, 0.0),
        id: *id,
        section: format!("level{id}"),
        guid: *guid,
      })
      .collect(),
    vertices: Vec::new(),
    edges: Vec::new(),
    points: Vec::new(),
    cross_tables,
  };

  graphs.write::<XRayByteOrder>(&mut graphs_writer).unwrap();

  chunk_bytes(
    SpawnGraphsChunk::CHUNK_ID,
    &graphs_writer.flush_raw_into_buffer().unwrap(),
  )
}

pub(crate) fn cross_table(level_guid: Uuid, game_guid: Uuid, nodes_count: u32) -> GraphCrossTable {
  GraphCrossTable {
    version: 10,
    nodes_count,
    vertices_count: 0,
    level_guid,
    game_guid,
    data: Vec::new(),
  }
}

/// Synthetic gamedata tree with a single valid `zaton` level and its game graph.
pub(crate) struct GamedataFixture {
  root: PathBuf,
  bundles: Vec<LevelBundleFixture>,
  spawn: Option<Vec<u8>>,
  declared_maps: Vec<String>,
  textures: Vec<String>,
}

impl GamedataFixture {
  pub(crate) fn new() -> Self {
    let unique: u64 = NEXT_TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);

    Self {
      root: build_absolute_generated_test_resource_path(&format!("levels/fixture-{unique}")),
      bundles: vec![LevelBundleFixture::valid("zaton")],
      spawn: Some(spawn_graph_bytes(
        &[("zaton", 108, ZATON_GUID)],
        vec![cross_table(ZATON_GUID, GRAPH_GUID, AI_NODE_COUNT)],
      )),
      declared_maps: vec![String::from("zaton")],
      textures: vec![String::from("map\\map_zaton"), String::from("level\\ground")],
    }
  }

  pub(crate) fn with_bundles(mut self, bundles: Vec<LevelBundleFixture>) -> Self {
    self.bundles = bundles;
    self
  }

  pub(crate) fn with_spawn(mut self, spawn: Option<Vec<u8>>) -> Self {
    self.spawn = spawn;
    self
  }

  pub(crate) fn with_declared_maps(mut self, declared_maps: Vec<String>) -> Self {
    self.declared_maps = declared_maps;
    self
  }

  pub(crate) fn without_texture(mut self, texture: &str) -> Self {
    self.textures.retain(|it| it != texture);
    self
  }

  pub(crate) fn verify(self) -> GamedataLevelsVerificationResult {
    let configs: PathBuf = self.root.join("configs");

    // A fixture describes a whole tree, so `without` and `without_texture` only mean what they say while nothing
    // survives an earlier occupant of this root.
    let _ = fs::remove_dir_all(&self.root);

    fs::create_dir_all(configs.join("$scheme")).unwrap();
    fs::write(configs.join("system.ltx"), "").unwrap();
    fs::write(
      configs.join("game_maps_single.ltx"),
      format!(
        "[level_maps_single]\n{}\n",
        self
          .declared_maps
          .iter()
          .map(|name| format!("{name} ="))
          .collect::<Vec<_>>()
          .join("\n")
      ),
    )
    .unwrap();

    for texture in &self.textures {
      let path: PathBuf = self
        .root
        .join("textures")
        .join(format!("{}.dds", texture.replace('\\', "/")));

      fs::create_dir_all(path.parent().unwrap()).unwrap();
      fs::write(path, [1]).unwrap();
    }

    for bundle in &self.bundles {
      bundle.write(&self.root);
    }

    if let Some(spawn) = &self.spawn {
      fs::create_dir_all(self.root.join("spawns")).unwrap();
      fs::write(self.root.join("spawns").join("all.spawn"), spawn).unwrap();
    }

    let project: GamedataProject = GamedataProject::open(&GamedataProjectReadOptions {
      root: self.root.clone(),
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    })
    .unwrap();

    project
      .verify_levels(&GamedataProjectVerifyOptions::default())
      .expect("Expected level verification to complete")
  }
}

/// A root that outlived its run once handed one test another test's tree, so `without` kept the very file it claims to
/// remove and the suite failed about one run in ten.
#[test]
fn a_fixture_ignores_what_an_earlier_occupant_left_in_its_root() {
  let fixture: GamedataFixture =
    GamedataFixture::new().with_bundles(vec![LevelBundleFixture::valid("zaton").without("level.geom")]);

  LevelBundleFixture::valid("zaton")
    .with("level.details", vec![1])
    .write(&fixture.root);

  assert_only_rule(&fixture.verify(), "levels.missing-file");
}

pub(crate) fn rule_ids(result: &GamedataLevelsVerificationResult) -> BTreeSet<String> {
  result
    .get_findings()
    .iter()
    .map(|finding| finding.rule_id().to_string())
    .collect()
}

pub(crate) fn assert_only_rule(result: &GamedataLevelsVerificationResult, rule: &str) {
  assert_eq!(
    rule_ids(result),
    BTreeSet::from([String::from(rule)]),
    "Expected only [{rule}], findings were: {:?}",
    result
      .get_findings()
      .iter()
      .map(|finding| format!("{}: {}", finding.rule_id(), finding.message()))
      .collect::<Vec<_>>()
  );
}
