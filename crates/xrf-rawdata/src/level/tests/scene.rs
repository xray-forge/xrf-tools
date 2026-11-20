use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{
  EditableBuildParams, EditableLevelCamera, EditableLevelFile, EditableLevelGuid, EditableLevelOptions,
  EditableLevelScene, EditableObjectPart, EditableSpawnPointObject, EditableWayObject,
};
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_job::{JobHandle, ProgressSink, RecordingSink};
use xrf_level::{
  LevelEnvModFile, LevelEnvModifier, LevelFogVolFile, LevelGameFile, LevelGameRPoint, LevelGameWay, LevelGameWayLink,
  LevelGameWayPoint,
};
use xrf_math::Vector3d;
use xrf_test_utils::{GeneratedResources, HostFilesystem};
use xrf_utils::HostLinkPolicy;

use crate::level::build::level_exported_previous::LevelExportedPrevious;
use crate::level::geometry::level_visual_geometry::LevelVisualGeometry;
use crate::level::level_scene_exporter::LevelSceneExporter;
use crate::level::parts::level_fog_volume_conversion::LevelFogVolumeConversion;
use crate::level::scene::level_decompile_options::LevelDecompileOptions;
use crate::level::scene::level_decompile_report::LevelDecompileReport;
use crate::level::scene::level_decompile_request::LevelDecompileRequest;
use crate::level::scene::level_scene_decompile::LevelSceneDecompile;
use crate::level::scene::level_scene_decompiler::LevelSceneDecompiler;
use crate::level::scene::level_scene_library_file::LevelSceneLibraryFile;
use crate::level::scene::level_scene_object_file::LevelSceneObjectFile;
use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;
use crate::level::tests::fixtures::GUID;
use crate::object::compile::tangent_method::TangentMethod;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;
use crate::object::decompile::object_decompiler::ObjectDecompiler;
use crate::provenance::level_scene_inference::LevelSceneInference;
use crate::provenance::level_scene_provenance::LevelSceneProvenance;
use crate::provenance::object_provenance::ObjectProvenance;

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
    &LevelDecompileOptions::new("test_level", "test", "1.0.0"),
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
      ("test_level.level", String::from("level.game")),
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
  assert!(decompiled.provenance.refused.is_empty());
  assert!(
    !decompiled
      .provenance
      .inferred
      .contains(&LevelSceneInference::LightBrightness),
    "a scene holding no light records no light inference"
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
fn reads_each_steps_files_once_and_records_what_the_main_file_came_from() -> XrfResult {
  let files: Vec<&str> = LevelSceneDecompiler::get_source_files();

  assert_eq!(
    files.iter().filter(|file| **file == "level").count(),
    1,
    "the bundle two steps read is read once"
  );

  for file in [
    "level.geom",
    "level.cform",
    "level.hom",
    "level.som",
    "build.lights",
    "level.ps_static",
    "level.snd_static",
    "level.snd_env",
    "level.fog_vol",
    "level.wallmarks",
    "level.game",
    "level.env_mod",
    "level.spawn",
    "level_rs.spawn",
  ] {
    assert!(files.contains(&file), "{file} is a step's");
  }

  let decompiled: LevelSceneDecompile = decompile(&level_directory("main_sources", 10.0)?)?;

  assert_eq!(
    decompiled.provenance.files[0].sources,
    ["level.game"],
    "the game modes come from level.game; the directory holds no build.lights for light qualities"
  );

  Ok(())
}

#[test]
fn writes_each_object_under_the_objects_root_after_its_sidecar() -> XrfResult {
  let mut decompiled: LevelSceneDecompile = decompile(&level_directory("objects", 10.0)?)?;
  let floor = [[
    Vector3d::new(0.0, 0.0, 0.0),
    Vector3d::new(0.0, 0.0, 1.0),
    Vector3d::new(1.0, 0.0, 0.0),
  ]];
  let object = ObjectDecompiler::decompile_level_geometry(&LevelVisualGeometry::of_triangles(
    "floor\\floor_tile",
    "default",
    &SurfaceDress::default(),
    &floor,
  ))?;
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "objects_scene"))?;
  let objects: PathBuf = output.join("objects");

  decompiled.objects.push(LevelSceneObjectFile {
    reference: String::from("levels\\test_level\\test_level_sector_0"),
    provenance: ObjectProvenance::new("level.geom", &[], "test", "1.0.0", object.inferred, object.tangents),
    object: object.object,
  });

  assert!(
    decompiled.get_existing_object_paths(&objects)?.is_empty(),
    "nothing written yet"
  );

  decompiled.write_to_path::<XRayByteOrder>(&output.join("test_level.level"), &objects, &output.join("textures"))?;

  let path: PathBuf = objects
    .join("levels")
    .join("test_level")
    .join("test_level_sector_0.object");

  assert_eq!(
    EditableObjectFile::read_from_path::<XRayByteOrder, _>(&path)?.write_to_bytes::<XRayByteOrder>()?,
    decompiled.objects[0].object.write_to_bytes::<XRayByteOrder>()?,
    "read back, the object saves the bytes it was written as"
  );
  assert_eq!(
    ObjectProvenance::read_beside(&path)?,
    Some(decompiled.objects[0].provenance.clone())
  );
  assert_eq!(
    decompiled.get_existing_object_paths(&objects)?,
    [ObjectProvenance::sidecar_path(&path), path],
    "a front-end asks about both before writing over them"
  );

  Ok(())
}

// A texture a library object's surface names, or an object a scene names, is data and may point anywhere.
#[test]
fn refuses_a_name_that_would_leave_its_root_and_writes_nothing() -> XrfResult {
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "escaping_names"))?;
  let level: PathBuf = output.join("scenes").join("test_level.level");
  let decompiled: LevelSceneDecompile = decompile(&level_directory("escaping", 10.0)?)?;

  for name in ["..\\..\\outside", "c:\\outside", "c:outside", ""] {
    let mut texture: LevelSceneDecompile = decompiled.clone();
    let mut object: LevelSceneDecompile = decompiled.clone();

    texture.textures.push(LevelSceneTextureFile {
      name: String::from(name),
      bytes: vec![0],
      is_library: false,
    });
    object.library.push(LevelSceneLibraryFile {
      reference: String::from(name),
      object: vec![0],
      thumbnail: None,
    });

    for decompiled in [texture, object] {
      assert!(
        decompiled
          .write_to_path::<XRayByteOrder>(&level, &output.join("objects"), &output.join("textures"))
          .is_err(),
        "'{name}' is refused"
      );
    }
  }

  assert_eq!(
    std::fs::read_dir(&output)?.count(),
    0,
    "nothing is written, not even the sidecar"
  );

  Ok(())
}

// A link below a root is the user's layout: followed by default, refused before the first write where the run says so.
#[test]
fn refuses_a_link_below_a_root_where_asked_and_writes_nothing() -> XrfResult {
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "linked_root"))?;
  let (elsewhere, objects) = (output.join("elsewhere"), output.join("objects"));
  let level: PathBuf = output.join("scenes").join("test_level.level");
  let mut decompiled: LevelSceneDecompile = decompile(&level_directory("linked", 10.0)?)?;

  std::fs::create_dir_all(&elsewhere)?;
  std::fs::create_dir_all(&objects)?;

  if !HostFilesystem::link_directory(&elsewhere, &objects.join("levels")) {
    return Ok(());
  }

  decompiled.library.push(LevelSceneLibraryFile {
    reference: String::from("levels\\test_level\\copied"),
    object: vec![0],
    thumbnail: None,
  });

  assert!(
    decompiled
      .write_to_path_with::<XRayByteOrder>(&level, &objects, &output.join("textures"), HostLinkPolicy::Refuse)
      .is_err()
  );
  assert!(
    !output.join("scenes").exists(),
    "nothing is written, not even the sidecar"
  );
  assert_eq!(std::fs::read_dir(&elsewhere)?.count(), 0);

  decompiled.write_to_path_with::<XRayByteOrder>(&level, &objects, &output.join("textures"), HostLinkPolicy::Follow)?;

  assert!(elsewhere.join("test_level").join("copied.object").is_file());

  Ok(())
}

// The earlier scene's stale parts go only once the write is sure to go ahead.
#[test]
fn refuses_a_link_before_removing_an_earlier_scenes_parts() -> XrfResult {
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "linked_forced"))?;
  let elsewhere: PathBuf = output.join("elsewhere");
  let stale: PathBuf = elsewhere.join("stale.part");

  std::fs::create_dir_all(&elsewhere)?;
  std::fs::create_dir_all(output.join("levels"))?;
  std::fs::write(&stale, b"earlier")?;

  // The part directory is a link to an earlier scene's parts, which the scene writes through.
  if !HostFilesystem::link_directory(&elsewhere, &output.join("levels").join("test_level")) {
    return Ok(());
  }

  let request: LevelDecompileRequest = LevelDecompileRequest {
    source: level_directory("linked_forced_level", 10.0)?,
    destination: output.join("levels").join("test_level.level"),
    textures: Some(output.join("textures")),
    is_forced: true,
    link_policy: HostLinkPolicy::Refuse,
    ..LevelDecompileRequest::default()
  };

  assert!(
    LevelSceneDecompiler::decompile_to_path::<XRayByteOrder>(&request, "test", "1.0.0", &JobHandle::inert()).is_err()
  );
  assert!(stale.is_file(), "the earlier scene's part is left");
  assert!(!request.destination.exists(), "nothing is written");

  Ok(())
}

#[test]
fn writes_the_scene_and_its_sidecar_and_reads_them_back() -> XrfResult {
  let decompiled: LevelSceneDecompile = decompile(&level_directory("written", 10.0)?)?;
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "written_scene"))?;
  let level_path: PathBuf = output.join("test_level.level");

  decompiled.write_to_path::<XRayByteOrder>(&level_path, &output.join("objects"), &output.join("textures"))?;

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
  let exported: BTreeMap<&str, Vec<u8>> =
    LevelSceneExporter::export::<XRayByteOrder>(&decompile(&directory)?.scene, &[])?;

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
    LevelSceneExporter::export::<XRayByteOrder>(&decompile(&directory)?.scene, &[])?;

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
  let exported: BTreeMap<&str, Vec<u8>> = LevelSceneExporter::export::<XRayByteOrder>(&scene, &[])?;

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

#[test]
fn writes_a_scene_for_a_front_end_and_nothing_once_cancelled() -> XrfResult {
  let level: PathBuf = level_directory("front_end", 10.0)?;
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "front_end_out"))?;
  let request: LevelDecompileRequest = LevelDecompileRequest {
    source: level,
    destination: output.join("levels").join("test_level.level"),
    ..LevelDecompileRequest::default()
  };
  let cancelled: JobHandle = JobHandle::inert();

  cancelled.cancel();

  let error = LevelSceneDecompiler::decompile_to_path::<XRayByteOrder>(&request, "test", "1.0.0", &cancelled)
    .expect_err("a cancelled decompile stops");

  assert!(matches!(error, XrfError::Cancelled { .. }), "{error}");
  assert!(!request.destination.exists(), "a cancelled decompile writes nothing");

  let sink: Arc<RecordingSink> = Arc::new(RecordingSink::default());
  let report: LevelDecompileReport = LevelSceneDecompiler::decompile_to_path::<XRayByteOrder>(
    &request,
    "test",
    "1.0.0",
    &JobHandle::with_interval(Arc::clone(&sink) as Arc<dyn ProgressSink>, Duration::ZERO),
  )?;

  assert!(request.destination.is_file());
  assert!(report.tools.iter().any(|tool| tool.name == "spawn"));
  assert!(
    sink
      .list_reported()
      .iter()
      .any(|progress| progress.detail.as_deref() == Some("Ways and spawn points")),
    "the last step is reported by name"
  );

  Ok(())
}

// The level's own objects are those under `levels\<name>\` named `<name>_`; the SDK names its own there alike, so a
// forced run removes only those a decompile's sidecar marks.
// The part directory may hold the user's own files beside the parts, which the editor never reads.
#[test]
fn removes_forced_only_the_stale_files_the_editor_reads_as_parts() -> XrfResult {
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "stale_parts_out"))?;
  let parts: PathBuf = output.join("levels").join("test_level");
  let request: LevelDecompileRequest = LevelDecompileRequest {
    source: level_directory("stale_parts", 10.0)?,
    destination: output.join("levels").join("test_level.level"),
    textures: Some(output.join("textures")),
    is_forced: true,
    ..LevelDecompileRequest::default()
  };

  std::fs::create_dir_all(&parts)?;
  std::fs::write(parts.join("old_tool.part1"), b"earlier")?;
  std::fs::write(parts.join("notes.txt"), b"the user's")?;

  LevelSceneDecompiler::decompile_to_path::<XRayByteOrder>(&request, "test", "1.0.0", &JobHandle::inert())?;

  assert!(!parts.join("old_tool.part1").exists(), "a stale part is removed");
  assert!(parts.join("notes.txt").is_file(), "the user's file is left");

  Ok(())
}

#[test]
fn owns_the_levels_objects_refused_unforced_and_removed_forced_where_no_longer_written() -> XrfResult {
  let level: PathBuf = level_directory("own_objects", 10.0)?;
  let output: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "own_objects_out"))?;
  let stale: PathBuf = output.join("objects/levels/test_level/Test_Level_Sector_9.object");
  let kept: PathBuf = output.join("objects/levels/test_level/authored.object");
  let authored: PathBuf = output.join("objects/levels/test_level/test_level_fence.object");
  let request: LevelDecompileRequest = LevelDecompileRequest {
    source: level,
    destination: output.join("levels").join("test_level.level"),
    ..LevelDecompileRequest::default()
  };

  std::fs::create_dir_all(output.join("levels"))?;
  std::fs::create_dir_all(output.join("objects/levels/test_level"))?;
  std::fs::write(&stale, b"earlier")?;
  std::fs::write(
    ObjectProvenance::sidecar_path(&stale),
    ObjectProvenance::new("level.geom", b"", "test", "1.0.0", Vec::new(), TangentMethod::default()).to_json()?,
  )?;
  std::fs::write(&kept, b"authored")?;
  std::fs::write(&authored, b"authored")?;

  let error: String =
    LevelSceneDecompiler::decompile_to_path::<XRayByteOrder>(&request, "test", "1.0.0", &JobHandle::inert())
      .expect_err("an object of the level's own is there")
      .to_string();

  assert!(error.contains("Test_Level_Sector_9.object"), "{error}");
  assert!(!request.destination.exists(), "a refusal writes nothing");

  let report: LevelDecompileReport = LevelSceneDecompiler::decompile_to_path::<XRayByteOrder>(
    &LevelDecompileRequest {
      is_forced: true,
      ..request
    },
    "test",
    "1.0.0",
    &JobHandle::inert(),
  )?;

  assert!(!stale.exists(), "the object no longer written is removed");
  assert!(
    !ObjectProvenance::sidecar_path(&stale).exists(),
    "its sidecar goes with it"
  );
  assert!(kept.is_file(), "an object not named as the level's own is left");
  assert!(
    authored.is_file(),
    "an object of the level's name with no sidecar is left"
  );
  assert!(
    report
      .removed
      .iter()
      .any(|path| path.ends_with("Test_Level_Sector_9.object")),
    "{:?}",
    report.removed
  );

  Ok(())
}
