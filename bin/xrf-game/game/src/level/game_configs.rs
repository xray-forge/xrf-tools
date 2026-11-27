use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_game_alife::AlifeSettings;
use xrf_game_simulation::SimulationConfigSource;
use xrf_ltx::{Ltx, LtxResolution};
use xrf_ltx_project::LtxProject;
use xrf_vfs::XrayLogicalPath;

/// The tree's configs as the game reads them: `configs\system.ltx` resolved once, and any other config resolved on
/// request, both in the tree's dialect, loose or archived alike.
pub struct GameConfigs {
  project: LtxProject,
  system: Arc<LtxResolution>,
}

impl GameConfigs {
  /// The config holding `[alife]`.
  const ALIFE_FILE: &'static str = "alife.ltx";

  /// The configs' `[alife]`, a mod's overrides applied as the dialect applies them; Call of Pripyat's where the tree
  /// holds none or it does not read.
  pub fn read_alife_settings(&self) -> AlifeSettings {
    match self.read_config(Self::ALIFE_FILE) {
      Ok(ltx) => AlifeSettings::read_ltx(&ltx),
      Err(error) => {
        log::warn!(
          "{} does not read ({error}); A-Life runs on Call of Pripyat's",
          Self::ALIFE_FILE
        );

        AlifeSettings::default()
      }
    }
  }

  /// The configs of `project`, its `system.ltx` resolved now.
  ///
  /// # Errors
  ///
  /// Returns why `system.ltx` does not resolve.
  pub fn open(project: LtxProject) -> XrfResult<Self> {
    let system: Arc<LtxResolution> = project.system_ltx()?;

    Ok(Self { project, system })
  }
}

impl SimulationConfigSource for GameConfigs {
  fn get_system(&self) -> &Ltx {
    &self.system.ltx
  }

  /// The config at `path`, refused where the tree holds none: a resolution of a missing root answers an empty config,
  /// which would read as a config that sets nothing.
  fn read_config(&self, path: &str) -> XrfResult<Ltx> {
    let logical: XrayLogicalPath = self.project.config_path(path)?;

    if self.project.search().find(logical.as_str())?.is_none() {
      return Err(XrfError::new_not_found_error(format!(
        "the tree holds no {}",
        logical.as_str()
      )));
    }

    Ok(self.project.read_resolution(&logical)?.ltx.clone())
  }
}
