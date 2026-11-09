use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{
  EditableBuildParams, EditableLevelCamera, EditableLevelFile, EditableLevelGuid, EditableLevelOptions,
  EditableLevelScene, EditableObjectPart, EditableSpawnPointObject, EditableWayObject,
};
use xrf_error::XrfResult;
use xrf_level::{
  LevelEnvModFile, LevelEnvModifier, LevelFogVolFile, LevelGameFile, LevelGameRPoint, LevelGameWay, LevelGameWayLink,
  LevelGameWayPoint,
};
use xrf_math::Vector3d;
use xrf_test_utils::GeneratedResources;

use crate::level::tests::GUID;
use crate::{
  LevelDecompileOptions, LevelExportedPrevious, LevelFogVolumeConversion, LevelSceneDecompile, LevelSceneDecompiler,
  LevelSceneExporter, LevelSceneInference, LevelSceneProvenance,
};

fn respawn_point(game_types: u16) -> LevelGameRPoint {
  LevelGameRPoint {
    position: Vector3d::new(12.25, 1.0, 11.625),
    rotation: Vector3d::new(0.0, -3.125, 0.0),
    team: 1,
    kind: 0,
    game_types,
    profile: String::new(),
  }
}

fn game() -> LevelGameFile {
  LevelGameFile {
    rpoints: vec![respawn_point(4), respawn_point(8)],
    ways: vec![LevelGameWay {
      version: EditableWayObject::VERSION as u16,
      name: String::from("test_patrol_walk"),
      kind: None,
      points: (0..2)
        .map(|index| LevelGameWayPoint {
          position: Vector3d::new(index as f32, 0.5, -2.0),
          flags: 0,
          name: format!("wp{index:02}"),
        })
        .collect(),
      links: vec![LevelGameWayLink {
        from: 0,
        to: 1,
        probability: 1.0,
      }],
    }],
  }
}

fn environment(radius: f32) -> LevelEnvModFile {
  LevelEnvModFile {
    version: EditableSpawnPointObject::VERSION,
    modifiers: vec![LevelEnvModifier {
      position: Vector3d::new(-4.5, 2.0, 7.25),
      radius,
      power: 1.0,
      far_plane: 300.0,
      fog_color: Vector3d::ZERO,
      fog_density: 0.75,
      ambient: Vector3d::ZERO,
      sky_color: Vector3d::ZERO,
      hemi_color: Vector3d::new(1.0, 1.0, 1.0),
      use_flags: Some(63),
    }],
  }
}

fn bytes_of(write: impl FnOnce(&mut ChunkWriter) -> XrfResult) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  write(&mut writer)?;

  Ok(writer.buffer)
}

/// A level directory under this test's resources holding `level.game`, a `level.env_mod` of the given radius, and the
/// `level.fog_vol` of no fog every build writes.
fn level_directory(name: &str, radius: f32) -> XrfResult<PathBuf> {
  let directory: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), name))?;
  let fog: LevelFogVolFile = LevelFogVolumeConversion::export(&EditableObjectPart::new(GUID, Vec::new()))?;

  std::fs::write(
    directory.join(LevelGameFile::FILE_NAME),
    bytes_of(|writer| game().write::<XRayByteOrder>(writer))?,
  )?;
  std::fs::write(
    directory.join(LevelEnvModFile::FILE_NAME),
    bytes_of(|writer| environment(radius).write::<XRayByteOrder>(writer))?,
  )?;
  std::fs::write(
    directory.join(LevelFogVolFile::FILE_NAME),
    bytes_of(|writer| fog.write::<XRayByteOrder>(writer))?,
  )?;

  Ok(directory)
}

fn decompile(directory: &Path) -> XrfResult<LevelSceneDecompile> {
  LevelSceneDecompiler::decompile::<XRayByteOrder>(
    directory,
    &LevelDecompileOptions {
      name: String::from("test_level"),
      tool: String::from("test"),
      tool_version: String::from("1.0.0"),
    },
  )
}

#[test]
fn decompiles_a_level_into_a_scene_of_its_parts() -> XrfResult {
  let decompiled: LevelSceneDecompile = decompile(&level_directory("parts", 10.0)?)?;
  let scene: &EditableLevelScene = &decompiled.scene;
  let options = scene.level.get_level_options()?;

  assert_eq!(
    scene.tools.iter().map(|tool| tool.name.as_str()).collect::<Vec<_>>(),
    ["fog_volume", "spawn", "way"]
  );
  assert_eq!(scene.level.get_version()?, 5);
  assert_eq!((options.level_path.as_str(), options.game_type), ("test_level", 12));
  assert_eq!(scene.level.get_build_params()?.weld_distance, 0.0);
  assert_eq!(
    decompiled
      .provenance
      .files
      .iter()
      .map(|file| (file.written.as_str(), file.sources.join(",")))
      .collect::<Vec<_>>(),
    [
      ("test_level.level", String::new()),
      ("test_level/fog_volume.part", String::from("level.fog_vol")),
      ("test_level/spawn.part", String::from("level.game,level.env_mod")),
      ("test_level/way.part", String::from("level.game")),
    ]
  );
  assert_eq!(
    decompiled
      .provenance
      .sources
      .iter()
      .map(|source| source.file.as_str())
      .collect::<Vec<_>>(),
    ["level.env_mod", "level.fog_vol", "level.game"]
  );
  assert_eq!(decompiled.provenance.tool, "test");
  assert!(
    decompiled
      .provenance
      .inferred
      .contains(&LevelSceneInference::EntitiesNotDecompiled)
  );

  Ok(())
}

#[test]
fn decompiling_the_same_level_twice_gives_the_same_scene() -> XrfResult {
  let directory: PathBuf = level_directory("twice", 10.0)?;
  let first: EditableLevelGuid = decompile(&directory)?.scene.level.get_guid()?;
  let other: EditableLevelGuid = decompile(&level_directory("other", 12.5)?)?.scene.level.get_guid()?;

  assert_eq!(decompile(&directory)?.scene, decompile(&directory)?.scene);
  assert_eq!(
    decompile(&level_directory("again", 10.0)?)?.scene.level.get_guid()?,
    first
  );
  assert_ne!(other, first, "a level of other bytes is another scene");

  Ok(())
}

#[test]
fn writes_the_scene_and_its_sidecar_and_reads_them_back() -> XrfResult {
  let decompiled: LevelSceneDecompile = decompile(&level_directory("written", 10.0)?)?;
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "written_scene"))?;
  let level_path: PathBuf = output.join("test_level.level");

  decompiled.write_to_path::<XRayByteOrder>(&level_path)?;

  let read: EditableLevelScene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&level_path)?;
  let sidecar: LevelSceneProvenance =
    serde_json::from_slice(&std::fs::read(output.join("test_level.level.xrf.json"))?)?;

  assert_eq!(read.level.write_to_bytes()?, decompiled.scene.level.write_to_bytes()?);
  assert_eq!(read.tools.len(), decompiled.scene.tools.len());

  for (ours, written) in read.tools.iter().zip(&decompiled.scene.tools) {
    assert_eq!(ours.name, written.name);
    assert_eq!(
      ours.parts[0].write_to_bytes::<XRayByteOrder>()?,
      written.parts[0].write_to_bytes::<XRayByteOrder>()?
    );
  }

  assert!(output.join("test_level").join("spawn.part").is_file());
  assert_eq!(sidecar, decompiled.provenance);

  Ok(())
}

#[test]
fn exports_the_scene_back_to_the_files_it_came_from() -> XrfResult {
  let directory: PathBuf = level_directory("exported", 10.0)?;
  let exported: BTreeMap<&str, Vec<u8>> = LevelSceneExporter::export::<XRayByteOrder>(&decompile(&directory)?.scene)?;

  assert_eq!(
    exported.keys().copied().collect::<Vec<_>>(),
    ["level.env_mod", "level.fog_vol", "level.game"]
  );

  for (name, bytes) in &exported {
    assert_eq!(bytes, &std::fs::read(directory.join(name))?, "{name} exports otherwise");
  }

  Ok(())
}

#[test]
fn writes_an_export_saying_what_each_file_replaced() -> XrfResult {
  let directory: PathBuf = level_directory("rewritten", 10.0)?;
  let mut exported: BTreeMap<&str, Vec<u8>> =
    LevelSceneExporter::export::<XRayByteOrder>(&decompile(&directory)?.scene)?;

  exported.insert("level.game", b"other".to_vec());
  std::fs::remove_file(directory.join("level.fog_vol"))?;

  let previous: Vec<(String, LevelExportedPrevious)> = LevelSceneExporter::write_to_directory(&exported, &directory)?
    .into_iter()
    .map(|file| (file.name, file.previous))
    .collect();

  assert_eq!(
    previous,
    [
      (String::from("level.env_mod"), LevelExportedPrevious::Same),
      (String::from("level.fog_vol"), LevelExportedPrevious::New),
      (String::from("level.game"), LevelExportedPrevious::Changed),
    ]
  );
  assert_eq!(std::fs::read(directory.join("level.game"))?, b"other");

  Ok(())
}

#[test]
fn refuses_to_write_an_export_beside_a_file_it_does_not_build() -> XrfResult {
  let directory: PathBuf = level_directory("stale", 10.0)?;
  let exported: BTreeMap<&str, Vec<u8>> = BTreeMap::from([("level.game", b"other".to_vec())]);
  let before: Vec<u8> = std::fs::read(directory.join("level.game"))?;
  let error: String = LevelSceneExporter::write_to_directory(&exported, &directory)
    .unwrap_err()
    .to_string();

  assert!(error.contains("level.env_mod"), "{error}");
  assert_eq!(
    std::fs::read(directory.join("level.game"))?,
    before,
    "a refusal writes nothing"
  );

  Ok(())
}

#[test]
fn a_scene_of_no_tool_exports_the_files_every_build_writes() -> XrfResult {
  let scene: EditableLevelScene = EditableLevelScene {
    level: EditableLevelFile::new(
      &GUID,
      &EditableLevelOptions::new("test_level", EditableLevelOptions::SINGLE_PLAYER),
      &EditableBuildParams::DEFAULT,
      &EditableLevelCamera::default(),
      "test",
      0,
    ),
    tools: Vec::new(),
  };
  let exported: BTreeMap<&str, Vec<u8>> = LevelSceneExporter::export::<XRayByteOrder>(&scene)?;

  assert_eq!(
    exported.keys().copied().collect::<Vec<_>>(),
    ["level.env_mod", "level.fog_vol", "level.game"]
  );
  assert_eq!(
    exported["level.env_mod"],
    bytes_of(|writer| {
      LevelEnvModFile {
        version: EditableSpawnPointObject::VERSION,
        modifiers: Vec::new(),
      }
      .write::<XRayByteOrder>(writer)
    })?
  );

  Ok(())
}

#[test]
fn refuses_a_directory_of_no_level_file() -> XrfResult {
  let directory: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "empty"))?;
  let error: String = decompile(&directory).unwrap_err().to_string();

  assert!(error.contains("holds none of the level files"), "{error}");

  Ok(())
}
