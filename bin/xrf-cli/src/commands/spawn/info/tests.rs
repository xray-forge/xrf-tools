use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use uuid::{Uuid, uuid};
use xrf_chunk::ChunkWriter;
use xrf_math::Vector3d;
use xrf_spawn::{
  GraphHeader, LevelSpawnFile, LevelSpawnObject, SpawnALifeSpawnsChunk, SpawnArtefactSpawnsChunk, SpawnFile,
  SpawnGraphsChunk, SpawnHeaderChunk, SpawnPatrolsChunk, XRayByteOrder,
};
use xrf_test_utils::GeneratedResources;

use crate::commands::spawn::info::InfoCommand;
use crate::core::command::command_testing::run_command_for_result;
use crate::core::command::{CommandError, CommandResult, GenericCommand};

fn run(path: &Path) -> CommandResult<Option<Value>> {
  run_command_for_result(
    &InfoCommand,
    &[
      String::from(InfoCommand.operation()),
      String::from("--path"),
      path.display().to_string(),
      String::from("--silent"),
      String::from("--json"),
    ],
  )
}

fn new_object(id: u32, section: &str, name: &str) -> LevelSpawnObject {
  LevelSpawnObject {
    id,
    // `M_SPAWN`, which every packet of a list opens with.
    net_action: 1,
    section: String::from(section),
    name: String::from(name),
    script_game_id: 0,
    script_rp: 0xFF,
    position: Vector3d::new(1.0, 2.0, 3.0),
    direction: Vector3d::ZERO,
    payload: vec![7; 12],
  }
}

fn new_root_spawn() -> SpawnFile {
  let guid: Uuid = uuid!("78e55023-10b1-426f-9247-bb680e5fe0b7");

  SpawnFile {
    header: SpawnHeaderChunk {
      version: 10,
      guid,
      graph_guid: guid,
      objects_count: 0,
      levels_count: 0,
    },
    alife_spawn: SpawnALifeSpawnsChunk { objects: Vec::new() },
    artefact_spawn: SpawnArtefactSpawnsChunk { nodes: Vec::new() },
    patrols: SpawnPatrolsChunk { patrols: Vec::new() },
    graphs: SpawnGraphsChunk {
      header: GraphHeader {
        version: 10,
        vertices_count: 0,
        edges_count: 0,
        points_count: 0,
        guid,
        levels_count: 0,
      },
      levels: Vec::new(),
      vertices: Vec::new(),
      edges: Vec::new(),
      points: Vec::new(),
      cross_tables: Vec::new(),
    },
  }
}

#[test]
fn reports_a_root_spawn_as_its_kind() -> CommandResult {
  let path: PathBuf = GeneratedResources::create_dir("spawn_info/root")?.join("all.spawn");

  new_root_spawn().write_to_path::<XRayByteOrder, _>(&path)?;

  let result: Value = run(&path)?.expect("a root spawn is reported");

  assert_eq!(result["kind"], "root");
  assert_eq!(result["guid"], "78e55023-10b1-426f-9247-bb680e5fe0b7");
  assert_eq!(result["graph"]["version"], 10);

  Ok(())
}

#[test]
fn reports_a_level_spawn_list_by_section() -> CommandResult {
  let path: PathBuf = GeneratedResources::create_dir("spawn_info/level")?.join("level.spawn");
  let list: LevelSpawnFile = LevelSpawnFile {
    objects: vec![
      new_object(0, "graph_point", "fake_graph_point"),
      new_object(1, "m_stalker", "esc_wolf"),
      new_object(2, "graph_point", "fake_exit_point"),
    ],
  };
  let mut writer: ChunkWriter = ChunkWriter::new();

  list.write::<XRayByteOrder>(&mut writer)?;
  fs::write(&path, writer.flush_raw_into_buffer()?)?;

  assert_eq!(
    run(&path)?.expect("a level spawn list is reported"),
    json!({
      "kind": "level",
      "sections": [
        { "name": "graph_point", "objects": 2 },
        { "name": "m_stalker", "objects": 1 },
      ],
    })
  );

  Ok(())
}

#[test]
fn refuses_a_file_of_neither_kind_naming_both() -> CommandResult {
  let path: PathBuf = GeneratedResources::create_dir("spawn_info/neither")?.join("container.spawn");

  // One chunk holding four bytes: a container, but neither a root's header nor a spawn packet.
  fs::write(&path, [7, 0, 0, 0, 4, 0, 0, 0, 1, 2, 3, 4])?;

  let error: CommandError = run(&path).expect_err("a file of neither kind is refused");

  assert_eq!(error.exit_code(), 1);
  assert!(
    error.message().contains("Expected a root spawn") && error.message().contains("or a level's spawn list"),
    "{}",
    error.message()
  );

  Ok(())
}
