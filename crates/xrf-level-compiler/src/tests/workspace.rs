//! The working directory the SDK's compilers run in: the `fsgame.ltx` they read there and the logs they leave.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::sdk::level_sdk_workspace::LevelSdkWorkspace;

#[test]
fn points_the_compilers_at_the_levels_the_game_data_and_the_textures() {
  let text: String = LevelSdkWorkspace::get_file_system(
    Path::new("C:/work/levels"),
    Path::new("C:\\Games\\CoP\\gamedata"),
    Path::new("C:/sdk/gamedata/textures\\"),
  );

  assert!(
    text.contains("$game_levels$ = true | false | C:\\work\\levels\\\r\n"),
    "{text}"
  );
  assert!(
    text.contains("$game_data$ = false | true | C:\\Games\\CoP\\gamedata\\\r\n"),
    "{text}"
  );
  assert!(
    text.contains("$game_textures$ = true | true | C:\\sdk\\gamedata\\textures\\\r\n"),
    "{text}"
  );
  assert!(text.contains("$level$ = false | false | $game_levels$"), "{text}");
  assert!(text.contains("$logs$ = true | false | $sdk_root$ | logs\\"), "{text}");
}

#[test]
fn reads_only_a_log_written_since_the_run_started() -> XrfResult {
  let directory: PathBuf =
    GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "reads_only_a_fresh_log"))?;
  let workspace: LevelSdkWorkspace = LevelSdkWorkspace::create(
    &directory,
    Path::new("C:/work/levels"),
    Path::new("C:/gamedata"),
    Path::new("C:/gamedata/textures"),
  )?;

  std::fs::write(directory.join("logs").join("xrlc_user.log"), "Build succesful\r\n")?;

  let since: SystemTime = SystemTime::now() + Duration::from_secs(60);

  assert_eq!(workspace.find_log("xrlc_", since), None);
  assert_eq!(workspace.get_last_log_line("xrlc_", since), None);
  assert_eq!(
    workspace.get_last_log_line("xrlc_", SystemTime::UNIX_EPOCH).as_deref(),
    Some("Build succesful")
  );

  Ok(())
}
