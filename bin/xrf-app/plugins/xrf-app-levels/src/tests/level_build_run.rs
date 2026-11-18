use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_level::{BuildCformFile, LevelDetailsFile, LevelLightsFile};
use xrf_level_compiler::{LevelCompileRequest, LevelCompileStep};
use xrf_rawdata::LevelBuildRequest;
use xrf_shaders::ShaderCompilerFile;
use xrf_test_utils::GeneratedResources;

use crate::rawdata::level_build_run::LevelBuildRun;

#[test]
fn compiles_the_directory_the_build_writes_forced_as_the_build_is() {
  let build: LevelBuildRequest = LevelBuildRequest {
    source: PathBuf::from("C:/rawdata/levels/labx8.level"),
    destination: PathBuf::from("C:/game/gamedata/levels/labx8"),
    is_forced: true,
    ..LevelBuildRequest::default()
  };
  let compile: LevelCompileRequest = LevelCompileRequest {
    level: PathBuf::from("named elsewhere"),
    gamedata: PathBuf::from("C:/game/gamedata"),
    textures: vec![PathBuf::from("C:/rawdata/../gamedata/textures")],
    ..LevelCompileRequest::default()
  };
  let run: LevelBuildRun = LevelBuildRun::new(build.clone(), Some(compile), Some(PathBuf::from("dxcompiler.dll")));
  let compile: LevelCompileRequest = run.compile.expect("a compile");

  assert_eq!(run.build, build, "the build is run as the form names it");
  assert_eq!(compile.level, PathBuf::from("C:/game/gamedata/levels/labx8"));
  assert!(compile.is_forced, "forced as the build is");
  assert_eq!(compile.dxc, Some(PathBuf::from("dxcompiler.dll")));
  assert_eq!(
    compile.textures,
    vec![PathBuf::from("C:/rawdata/../gamedata/textures")],
    "the roots named are kept"
  );
}

/// A compile the build cannot feed, or one that would refuse its directory, refuses the run before the build writes.
#[test]
fn refuses_the_whole_run_before_writing_where_the_compile_would_refuse() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "refused"))?;
  let destination: PathBuf = root.join("levels").join("test_level");
  let gamedata: PathBuf = root.join("gamedata");
  let build: LevelBuildRequest = LevelBuildRequest {
    source: root.join("scenes").join("test_level.level"),
    destination: destination.clone(),
    game_materials: Some(gamedata.join("gamemtl.xr")),
    ..LevelBuildRequest::default()
  };
  let details: LevelCompileRequest = LevelCompileRequest {
    steps: vec![LevelCompileStep::Details],
    gamedata: gamedata.clone(),
    ..LevelCompileRequest::default()
  };
  let refuse = |build: LevelBuildRequest| -> String {
    LevelBuildRun::new(build, Some(details.clone()), None)
      .run(&JobHandle::inert())
      .expect_err("the run is refused")
      .to_string()
  };

  std::fs::create_dir_all(&destination)?;
  std::fs::create_dir_all(&gamedata)?;
  std::fs::write(gamedata.join(ShaderCompilerFile::FILE_NAME), b"")?;
  // An earlier compile's details, and the inputs the details read that the build does not write.
  std::fs::write(destination.join(LevelDetailsFile::FILE_NAME), b"kept")?;
  std::fs::write(destination.join(BuildCformFile::FILE_NAME), b"")?;
  std::fs::write(destination.join(LevelLightsFile::FILE_NAME), b"")?;

  assert!(
    refuse(LevelBuildRequest {
      game_materials: None,
      ..build.clone()
    })
    .contains("name them")
  );
  assert!(refuse(build).contains(LevelDetailsFile::FILE_NAME));
  assert_eq!(std::fs::read(destination.join(LevelDetailsFile::FILE_NAME))?, b"kept");
  assert!(!destination.join("level.game").exists(), "a refusal writes nothing");

  Ok(())
}

#[test]
fn builds_alone_where_no_step_is_chosen() {
  assert!(
    LevelBuildRun::new(LevelBuildRequest::default(), None, None)
      .compile
      .is_none()
  );
}
