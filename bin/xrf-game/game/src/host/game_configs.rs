use xrf_error::{XrfError, XrfResult};
use xrf_game_alife::AlifeSettings;
use xrf_game_config::{ConfigFile, ConfigIssues, ConfigSource};
use xrf_game_movement::WalkMode;
use xrf_ltx_project::LtxProject;
use xrf_vfs::{XrayLogicalPath, XraySearch};

use crate::host::game_alife_config::GameAlifeConfig;
use crate::host::game_player_config::GamePlayerConfig;
use crate::input::game_controls::GameControls;
use crate::time::game_clock::GameClock;

/// The tree's configs as the game reads them, kept for the run (plan decision 25): `configs\system.ltx` resolved
/// once, and any other config resolved on request, both in the tree's dialect, loose or archived alike. Every system
/// reads them through `xrf-game-config`, and what does not read is collected as issues, never a silent default.
pub struct GameConfigs {
  project: LtxProject,
  system: ConfigFile,
}

impl GameConfigs {
  /// The config holding `[alife]`.
  const ALIFE_FILE: &'static str = "alife.ltx";

  const HUD_SOUND_SECTION: &'static str = "hud_sound";
  const STEP_VOLUME_KEY: &'static str = "hud_step_sound_vol_k";

  /// The path `system.ltx` is named by in issues.
  const SYSTEM_FILE: &'static str = "system.ltx";

  /// The configs of `project`, its `system.ltx` resolved now.
  ///
  /// # Errors
  ///
  /// Returns why `system.ltx` does not resolve.
  pub fn open(project: LtxProject) -> XrfResult<Self> {
    let system: ConfigFile = ConfigFile::of_resolution(Self::SYSTEM_FILE, project.system_ltx()?);

    Ok(Self { project, system })
  }

  /// The configs' `[alife]`, a mod's overrides applied as the dialect applies them: A-Life's settings and the game's
  /// clock, Call of Pripyat's for what does not read, each an issue in `issues`.
  pub(crate) fn read_alife(&self, issues: &mut ConfigIssues) -> GameAlifeConfig {
    let defaults: GameAlifeConfig = GameAlifeConfig {
      settings: AlifeSettings::default(),
      clock: GameClock::default(),
    };
    let file: ConfigFile = match self.read_file(Self::ALIFE_FILE) {
      Ok(file) => file,
      Err(error) => {
        issues.push_file(
          Self::ALIFE_FILE,
          format!("does not read, A-Life runs on Call of Pripyat's: {error}"),
        );

        return defaults;
      }
    };

    file
      .require_section(AlifeSettings::SECTION, issues)
      .map_or(defaults, |mut section| GameAlifeConfig {
        settings: AlifeSettings::read(&mut section),
        clock: GameClock::read(&mut section),
      })
  }

  /// What `system.ltx` sets for the player: how they walk, from `[actor]`, and how loud their own steps play,
  /// `[hud_sound] hud_step_sound_vol_k` where it is set, as `InitHudSoundSettings` reads it, 1 where it is not.
  pub(crate) fn read_player_config(&self, issues: &mut ConfigIssues) -> GamePlayerConfig {
    let walk: WalkMode = self
      .system
      .require_section(WalkMode::ACTOR_SECTION, issues)
      .map_or_else(WalkMode::default, |mut section| WalkMode::read(&mut section));
    let step_volume: f32 = self
      .system
      .read_section(Self::HUD_SOUND_SECTION, issues)
      .map_or(1.0, |mut section| section.read_or(Self::STEP_VOLUME_KEY, 1.0));

    GamePlayerConfig { walk, step_volume }
  }

  /// The tree's own controls, its `default_controls.ltx`; Call of Pripyat's where the tree holds none.
  ///
  /// # Errors
  ///
  /// Returns why the file the tree holds cannot be read.
  pub(crate) fn read_controls(&self, issues: &mut ConfigIssues) -> XrfResult<GameControls> {
    let logical: XrayLogicalPath = self.project.config_path(GameControls::FILE)?;
    let search: &XraySearch = self.project.search();

    if search.find(logical.as_str())?.is_none() {
      log::info!(
        "The tree holds no {}; the controls are Call of Pripyat's",
        GameControls::FILE
      );

      return Ok(GameControls::default());
    }

    let bytes: Vec<u8> = search.read(logical.as_str())?;

    Ok(GameControls::read(&String::from_utf8_lossy(&bytes), issues))
  }
}

impl ConfigSource for GameConfigs {
  fn get_system(&self) -> &ConfigFile {
    &self.system
  }

  /// The config at `path`, refused where the tree holds none: a resolution of a missing root answers an empty config,
  /// which would read as a config that sets nothing.
  fn read_file(&self, path: &str) -> XrfResult<ConfigFile> {
    let logical: XrayLogicalPath = self.project.config_path(path)?;

    if self.project.search().find(logical.as_str())?.is_none() {
      return Err(XrfError::new_not_found_error(format!(
        "the tree holds no {}",
        logical.as_str()
      )));
    }

    Ok(ConfigFile::of_resolution(path, self.project.read_resolution(&logical)?))
  }
}
