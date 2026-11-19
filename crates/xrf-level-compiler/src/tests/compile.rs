use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_level::BuildPrjFile;
use xrf_shaders::ShaderCompilerFile;
use xrf_test_utils::GeneratedResources;

use crate::tests::geometry::geometry_mesh::project;
use crate::{AiMapCovers, LevelCompileRequest, LevelCompileStep, LevelCompiler};

fn level(name: &str, files: &[&str]) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), name))?;
  let level: PathBuf = root.join("levels").join("test_level");

  std::fs::create_dir_all(&level)?;

  for file in files {
    std::fs::write(level.join(file), b"")?;
  }

  Ok(level)
}

// An empty compiler shader library: the checks only need to find it.
fn gamedata(name: &str) -> XrfResult<PathBuf> {
  let root: PathBuf = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), name))?;

  std::fs::write(root.join(ShaderCompilerFile::FILE_NAME), b"")?;

  Ok(root)
}

fn request(level: &Path, steps: &[LevelCompileStep]) -> LevelCompileRequest {
  LevelCompileRequest {
    level: level.to_path_buf(),
    steps: steps.to_vec(),
    textures: vec![level.join("textures")],
    ..LevelCompileRequest::default()
  }
}

#[test]
fn runs_each_step_once_in_its_order_and_every_step_when_none_is_named() {
  assert_eq!(
    LevelCompiler::get_steps(&[LevelCompileStep::Ai, LevelCompileStep::Geometry, LevelCompileStep::Ai]),
    [LevelCompileStep::Geometry, LevelCompileStep::Ai]
  );
  assert_eq!(LevelCompiler::get_steps(&[]), LevelCompileStep::ALL);
  assert_eq!(LevelCompileStep::from_name("light"), Some(LevelCompileStep::Light));
  assert_eq!(LevelCompileStep::from_name("lights"), None);
}

#[test]
fn searches_the_game_data_textures_after_the_roots_named_and_once() {
  let game: PathBuf = PathBuf::from("C:/game/gamedata/textures");
  let roots = |textures: Vec<PathBuf>| {
    LevelCompileRequest {
      gamedata: PathBuf::from("C:/game/gamedata"),
      textures,
      ..LevelCompileRequest::default()
    }
    .get_textures_roots()
  };
  let strips: PathBuf = PathBuf::from("C:/rawdata/../gamedata/textures");

  assert_eq!(roots(Vec::new()), std::slice::from_ref(&game));
  assert_eq!(roots(vec![strips.clone()]), [strips, game.clone()]);
  assert_eq!(roots(vec![game.clone()]), [game]);
}

#[test]
fn refuses_the_light_step_without_the_geometry_step_before_running_anything() -> XrfResult {
  let level: PathBuf = level("light_alone", &["build.prj"])?;
  let error: String = LevelCompiler::compile(
    &request(&level, &[LevelCompileStep::Light, LevelCompileStep::Details]),
    &JobHandle::inert(),
  )
  .unwrap_err()
  .to_string();

  // The light is baked in the geometry's pass, so it has nothing to light alone.
  assert!(error.contains("light step runs only with the geometry step"), "{error}");

  Ok(())
}

#[test]
fn refuses_a_step_whose_input_is_missing_before_running_anything() -> XrfResult {
  let level: PathBuf = level("missing_input", &["build.prj", "build.details"])?;
  let error: String = LevelCompiler::compile(&request(&level, &[LevelCompileStep::Details]), &JobHandle::inert())
    .unwrap_err()
    .to_string();

  assert!(error.contains("details step needs build.cform"), "{error}");

  Ok(())
}

#[test]
fn takes_what_the_geometry_writes_as_given_when_it_runs_first() -> XrfResult {
  // Chained, the checks pass the library and fail reading the empty build.prj rather than on a missing input.
  let level: PathBuf = level("chained", &["build.prj", "build.aimap"])?;
  let gamedata: PathBuf = gamedata("chained_gamedata")?;
  let compile = |steps: &[LevelCompileStep]| {
    LevelCompiler::compile(
      &LevelCompileRequest {
        gamedata: gamedata.clone(),
        ..request(&level, steps)
      },
      &JobHandle::inert(),
    )
    .unwrap_err()
    .to_string()
  };
  let alone: String = compile(&[LevelCompileStep::Ai]);
  let chained: String = compile(&[LevelCompileStep::Geometry, LevelCompileStep::Ai]);

  assert!(alone.contains("ai step needs build.cform"), "{alone}");
  assert!(
    !chained.contains("needs build.cform") && !chained.contains(ShaderCompilerFile::FILE_NAME),
    "{chained}"
  );

  Ok(())
}

#[test]
fn refuses_the_geometry_step_without_the_compiler_shader_library() -> XrfResult {
  let level: PathBuf = level("no_library", &["build.prj"])?;
  let error: String = LevelCompiler::compile(&request(&level, &[LevelCompileStep::Geometry]), &JobHandle::inert())
    .unwrap_err()
    .to_string();

  // The geometry compiler reads each face's flags from the library.
  assert!(
    error.contains("geometry step needs") && error.contains("shaders_xrlc.xr"),
    "{error}"
  );

  Ok(())
}

#[test]
fn counts_every_file_a_step_writes_and_every_earlier_page_as_the_compiles() -> XrfResult {
  let level: PathBuf = level("outputs", &["Level.Geom", "LMAP#3_2.dds", "build.prj", "level.details"])?;
  let found = |steps: &[LevelCompileStep], implicit: &[String]| -> XrfResult<Vec<String>> {
    Ok(
      LevelCompiler::get_outputs(&request(&level, steps), steps, implicit)
        .find_existing()?
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect(),
    )
  };

  std::fs::create_dir_all(level.join("terrain"))?;
  std::fs::write(level.join("terrain").join("terrain_Lm.dds"), b"")?;

  // The geometry writes over its own files and every page an earlier compile left, in any case; an input is not one.
  assert_eq!(
    found(&[LevelCompileStep::Geometry], &[])?,
    ["Level.Geom", "LMAP#3_2.dds"]
  );
  // The light step writes an implicitly lit texture's maps below the level.
  assert_eq!(
    found(
      &[LevelCompileStep::Geometry, LevelCompileStep::Light],
      &[String::from("terrain\\terrain")]
    )?,
    ["Level.Geom", "terrain_Lm.dds", "LMAP#3_2.dds"]
  );
  assert_eq!(found(&[LevelCompileStep::Ai], &[])?, Vec::<String>::new());
  assert_eq!(found(&[LevelCompileStep::Details], &[])?, ["level.details"]);

  Ok(())
}

#[test]
fn checks_a_build_by_what_it_writes_before_it_writes() -> XrfResult {
  let level: PathBuf = level("build_checked", &[])?;
  let gamedata: PathBuf = gamedata("build_checked_gamedata")?;
  let project: BuildPrjFile = project(&[], &[])?;
  let check = |steps: &[LevelCompileStep], written: &[&str]| {
    LevelCompiler::check_build(
      &LevelCompileRequest {
        gamedata: gamedata.clone(),
        ..request(&level, steps)
      },
      &project,
      written,
      &["build.details", "build.prj"],
    )
  };

  // An earlier build's details are gone once a build of none is written.
  std::fs::write(level.join("build.details"), b"")?;

  let details: String = check(&[LevelCompileStep::Details], &["build.prj"])
    .unwrap_err()
    .to_string();
  let textures: String = check(&[LevelCompileStep::Geometry], &["build.prj"])
    .unwrap_err()
    .to_string();

  assert!(
    details.contains("details step needs build.details, which the build does not write"),
    "{details}"
  );
  assert!(
    textures.contains("have no .thm") && textures.contains("ground"),
    "{textures}"
  );

  std::fs::create_dir_all(level.join("textures"))?;
  std::fs::write(level.join("textures").join("ground.thm"), b"")?;

  check(&[LevelCompileStep::Geometry], &["build.prj"])
}

#[test]
fn needs_no_ray_model_for_an_ai_map_with_no_covers() -> XrfResult {
  let level: PathBuf = level("ai_no_covers", &["build.aimap"])?;
  let refuse = |covers: AiMapCovers| {
    LevelCompiler::compile(
      &LevelCompileRequest {
        ai_covers: covers,
        ..request(&level, &[LevelCompileStep::Ai])
      },
      &JobHandle::inert(),
    )
    .unwrap_err()
    .to_string()
  };

  assert!(refuse(AiMapCovers::Full).contains("ai step needs build.prj"));
  // With no covers the checks pass, no library needed, and the empty build.aimap fails to read.
  assert!(!refuse(AiMapCovers::None).contains("needs"));

  Ok(())
}
