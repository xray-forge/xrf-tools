use std::path::PathBuf;

use xrf_level_compiler::LevelCompileRequest;
use xrf_rawdata::LevelBuildRequest;

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

#[test]
fn builds_alone_where_no_step_is_chosen() {
  assert!(
    LevelBuildRun::new(LevelBuildRequest::default(), None, None)
      .compile
      .is_none()
  );
}
