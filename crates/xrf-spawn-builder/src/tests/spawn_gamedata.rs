use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;
use xrf_job::JobHandle;
use xrf_test_utils::GeneratedResources;

use crate::{SpawnListedLevel, SpawnRootGamedata, SpawnRootOptions, SpawnRootRequest};

fn gamedata(case: &str) -> XrfResult<(PathBuf, PathBuf)> {
  let root: PathBuf = GeneratedResources::create_dir(&format!("spawn-builder/gamedata/{case}"))?;
  let (gamedata, levels) = (root.join("gamedata"), root.join("built"));

  fs::create_dir_all(gamedata.join("configs"))?;
  fs::write(
    gamedata.join("configs/game_levels.ltx"),
    "[levels]\nlevel108\nlevel150\n\n[level108]\nname = zaton\ncaption = \"st_zaton\"\noffset = 0, 0, 0\nid = 108\n\n\
     [level150]\nname = labx8\noffset = 0, 0, 0\nid = 150\n",
  )?;

  for (level, files) in [
    ("labx8", ["level.ai", "level.spawn", "level.game"].as_slice()),
    ("zaton", ["level.ai", "level.spawn"].as_slice()),
  ] {
    fs::create_dir_all(levels.join(level))?;

    for file in files {
      fs::write(levels.join(level).join(file), b"")?;
    }
  }

  Ok((gamedata, levels))
}

fn request(gamedata: &Path, levels: Option<&[&str]>, name: &str) -> SpawnRootRequest {
  SpawnRootRequest {
    gamedata: gamedata.to_path_buf(),
    levels_root: None,
    levels: levels.map(|levels| levels.iter().map(|level| (*level).to_owned()).collect()),
    destination: None,
    name: name.to_owned(),
    options: SpawnRootOptions::default(),
    is_forced: false,
  }
}

fn assert_refused(request: &SpawnRootRequest, message: &str) {
  let error: String = SpawnRootGamedata::build(request, &JobHandle::inert())
    .expect_err("expect the build refused")
    .to_string();

  assert!(error.contains(message), "Unexpected error: {error}");
}

#[test]
fn reads_the_levels_and_writes_the_spawn_under_the_gamedata_root_unless_told_otherwise() {
  let mut request: SpawnRootRequest = request(Path::new("D:\\game\\gamedata"), None, "qa_levels");

  assert_eq!(
    request.get_levels_root(),
    Path::new("D:\\game\\gamedata").join("levels")
  );
  assert_eq!(
    request.get_spawn_path(),
    Path::new("D:\\game\\gamedata").join("spawns").join("qa_levels.spawn")
  );

  request.levels_root = Some(PathBuf::from("D:\\qa\\gamedata\\levels"));
  request.destination = Some(PathBuf::from("D:\\qa\\spawn"));

  assert_eq!(request.get_levels_root(), Path::new("D:\\qa\\gamedata\\levels"));
  assert_eq!(
    request.get_spawn_path(),
    Path::new("D:\\qa\\spawn").join("qa_levels.spawn")
  );
}

#[test]
fn lists_the_levels_the_game_lists_marking_the_ones_built() -> XrfResult {
  let (gamedata, levels) = gamedata("listed")?;

  assert_eq!(
    SpawnRootGamedata::list_levels(&gamedata, &levels)?,
    [
      SpawnListedLevel {
        name: String::from("zaton"),
        caption: String::from("st_zaton"),
        id: 108,
        is_complete: false,
      },
      SpawnListedLevel {
        name: String::from("labx8"),
        caption: String::from("labx8"),
        id: 150,
        is_complete: true,
      },
    ]
  );

  // Under the gamedata root itself no level is built.
  assert!(
    SpawnRootGamedata::list_levels(&gamedata, &gamedata.join("levels"))?
      .iter()
      .all(|level| !level.is_complete)
  );

  Ok(())
}

#[test]
fn refuses_levels_it_cannot_take_before_reading_any() -> XrfResult {
  let (gamedata, levels) = gamedata("refused")?;
  let at = |names: &[&str]| SpawnRootRequest {
    levels_root: Some(levels.clone()),
    ..request(&gamedata, Some(names), "qa_levels")
  };

  assert_refused(&at(&[]), "No level is asked for");

  for name in ["", "..", "spawns/qa_levels"] {
    assert_refused(
      &SpawnRootRequest {
        name: name.to_owned(),
        ..at(&["labx8"])
      },
      "is not a file name",
    );
  }

  assert_refused(
    &at(&["jupiter"]),
    "Level jupiter is not one configs/game_levels.ltx lists",
  );
  assert_refused(&at(&["labx8", "LABX8"]), "Level LABX8 is asked for twice");
  // Every listed level under the gamedata root lacks its files.
  assert_refused(
    &request(&gamedata, None, "qa_levels"),
    "No level the game lists holds level.ai, level.spawn and level.game",
  );

  Ok(())
}

#[test]
fn refuses_to_write_over_a_spawn_in_any_case_of_its_name_unless_forced() -> XrfResult {
  let (gamedata, levels) = gamedata("game-spawn")?;
  let destination: PathBuf = gamedata.join("spawns");

  fs::create_dir_all(&destination)?;
  fs::write(destination.join("All.spawn"), b"shipped")?;
  fs::write(destination.join("test.spawn"), b"earlier")?;

  for name in ["all", "ALL", "test"] {
    let mut request: SpawnRootRequest = SpawnRootRequest {
      levels_root: Some(levels.clone()),
      ..request(&gamedata, Some(&["labx8"]), name)
    };

    assert_refused(&request, "force it to replace them");

    // Forced, it goes on to read the levels, whose files here are empty.
    request.is_forced = true;
    assert_refused(&request, "Level labx8:");
  }

  assert_eq!(fs::read(destination.join("All.spawn"))?, b"shipped");
  assert_eq!(fs::read(destination.join("test.spawn"))?, b"earlier");

  Ok(())
}
