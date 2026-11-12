use std::path::{Path, PathBuf};
use std::time::SystemTime;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// The directory the SDK's compilers run in. They read `fsgame.ltx` from their working directory, `$fs_root$` being
/// that directory, and write their logs under `$logs$`; this writes one naming the level's parent as `$game_levels$`,
/// the game data's libraries as `$game_data$` and the textures root as `$game_textures$`.
pub(crate) struct LevelSdkWorkspace {
  pub(crate) directory: PathBuf,
}

impl LevelSdkWorkspace {
  /// The name the compilers read their file system from.
  const FILE_SYSTEM: &'static str = "fsgame.ltx";

  /// The workspace in `directory`, its `fsgame.ltx` written for `levels`, `gamedata` and `textures`.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory or the file cannot be written.
  pub(crate) fn create(directory: &Path, levels: &Path, gamedata: &Path, textures: &Path) -> XrfResult<Self> {
    std::fs::create_dir_all(directory.join("logs")).map_err(|error| {
      XrfError::new_io_error(
        format!("{} was not created: {error}", format_path(directory)),
        error.kind(),
      )
    })?;
    std::fs::write(
      directory.join(Self::FILE_SYSTEM),
      Self::get_file_system(levels, gamedata, textures),
    )
    .map_err(|error| {
      XrfError::new_io_error(
        format!("{} was not written: {error}", format_path(directory)),
        error.kind(),
      )
    })?;

    Ok(Self {
      directory: directory.to_path_buf(),
    })
  }

  /// `fsgame.ltx` as SDK 0.7's compilers read it: `recursive | notify | root | add`, each root a directory ending in
  /// a backslash.
  pub(crate) fn get_file_system(levels: &Path, gamedata: &Path, textures: &Path) -> String {
    let root = |path: &Path| {
      let mut text: String = path.display().to_string().replace('/', "\\");

      if !text.ends_with('\\') {
        text.push('\\');
      }

      text
    };
    let (levels, gamedata, textures) = (root(levels), root(gamedata), root(textures));

    [
      String::from("$app_data_root$ = true | false | $fs_root$ | _appdata_\\"),
      String::from("$sdk_root$ = false | false | $fs_root$"),
      format!("$game_data$ = false | true | {gamedata}"),
      String::from("$game_ai$ = true | false | $game_data$ | ai\\"),
      String::from("$game_spawn$ = true | false | $game_data$ | spawns\\"),
      format!("$game_levels$ = true | false | {levels}"),
      String::from("$game_meshes$ = true | true | $game_data$ | meshes\\ | *.ogf;*.omf | Game Object files"),
      String::from("$game_dm$ = true | true | $game_data$ | meshes\\ | *.dm | Detail Model files"),
      String::from("$game_shaders$ = true | true | $game_data$ | shaders\\"),
      format!("$game_textures$ = true | true | {textures}"),
      String::from("$game_config$ = true | false | $game_data$ | configs\\"),
      format!("$textures$ = true | true | {textures}"),
      String::from("$level$ = false | false | $game_levels$"),
      String::from("$logs$ = true | false | $sdk_root$ | logs\\"),
      String::new(),
    ]
    .join("\r\n")
  }

  /// The text of the last line the newest log beginning with `prefix` written since `since` holds, a compiler's
  /// progress so far.
  pub(crate) fn get_last_log_line(&self, prefix: &str, since: SystemTime) -> Option<String> {
    let log: PathBuf = self.find_log(prefix, since)?;
    let text: String = String::from_utf8_lossy(&std::fs::read(log).ok()?).into_owned();

    text
      .lines()
      .rev()
      .map(|line| line.trim_start_matches(['|', ' ', '*']).trim())
      .find(|line| !line.is_empty())
      .map(str::to_owned)
  }

  /// The newest log in the workspace's `logs` beginning with `prefix`, the compiler's own (`xrlc_<user>.log`), written
  /// since `since`: a workspace is reused, and an older run's log says nothing of this one.
  pub(crate) fn find_log(&self, prefix: &str, since: SystemTime) -> Option<PathBuf> {
    std::fs::read_dir(self.directory.join("logs"))
      .ok()?
      .filter_map(|entry| entry.ok())
      .filter(|entry| {
        entry.file_name().to_string_lossy().to_lowercase().starts_with(prefix)
          && entry
            .path()
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("log"))
      })
      .filter_map(|entry| {
        Some((
          entry.metadata().and_then(|metadata| metadata.modified()).ok()?,
          entry.path(),
        ))
      })
      .filter(|(modified, _)| *modified >= since)
      .max_by_key(|(modified, _)| *modified)
      .map(|(_, path)| path)
  }
}
