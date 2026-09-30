//! Reading every cycle, effect and definitions config of an environment read, each as its engine reads it.

use std::collections::HashSet;
use std::sync::Arc;

use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_ltx::Section;
use xrf_vfs::XrayLogicalPath;

use crate::ambient::{Ambient, AmbientKey, LevelAmbients, SoundChannel, SoundChannelKey};
use crate::catalog::environment_config::EnvironmentConfig;
use crate::catalog::environment_read_pass::EnvironmentReadPass;
use crate::finding::{EnvironmentFinding, EnvironmentRule};
use crate::level::WeatherGraphs;
use crate::section::EnvironmentSectionReader;
use crate::sun::SunTable;
use crate::thunderbolt::{ThunderboltSettings, ThunderboltSettingsKey};
use crate::weather::{WeatherCycle, WeatherCycleId, WeatherCycleKind};

impl EnvironmentReadPass<'_> {
  /// Every cycle or effect in its directory, a config the dialect attaches to another left out.
  pub(super) fn read_cycles(&mut self, kind: WeatherCycleKind) -> XrfResult<Vec<WeatherCycle>> {
    let directory: &str = Self::directory_of(kind);
    let files: Vec<XrayLogicalPath> = self.list_configs(directory)?;

    // `R_ASSERT2(!WeatherCycles.empty(), "Empty weathers.")`.
    if kind == WeatherCycleKind::Cycle && files.is_empty() {
      self.findings.push(EnvironmentFinding {
        file: self.project.config_path(directory)?.as_str().to_owned(),
        key: None,
        message: format!("No weather cycles under {directory}, where the engine requires at least one"),
        rule: EnvironmentRule::Engine,
        section: None,
      });
    }

    let mut cycles: Vec<WeatherCycle> = Vec::with_capacity(files.len());

    for file in files {
      self.options.job.check_cancelled()?;

      let Some(config) = self.open(&file)? else {
        continue;
      };
      let id: WeatherCycleId = WeatherCycleId {
        kind,
        name: Self::stem_of(&file).to_owned(),
      };
      let mut reader: EnvironmentSectionReader = self.reader(kind.get_subject(), &config);

      cycles.push(WeatherCycle::read(&mut reader, &id, config.get_ltx()));
    }

    Ok(cycles)
  }

  /// Every section of one definitions config, read by its kind's reader; none where the config is not there.
  pub(super) fn read_definitions<T>(
    &mut self,
    relative: &str,
    subject: &'static str,
    read: impl Fn(&mut EnvironmentSectionReader, &str, &Section) -> T,
  ) -> XrfResult<Vec<T>> {
    self.options.job.check_cancelled()?;

    let path: XrayLogicalPath = self.project.config_path(relative)?;
    let Some(config) = self.open(&path)? else {
      return Ok(Vec::new());
    };

    Ok(self.read_sections(&config, subject, read))
  }

  pub(super) fn read_sections<T>(
    &mut self,
    config: &EnvironmentConfig,
    subject: &'static str,
    read: impl Fn(&mut EnvironmentSectionReader, &str, &Section) -> T,
  ) -> Vec<T> {
    let mut reader: EnvironmentSectionReader = self.reader(subject, config);

    config
      .get_ltx()
      .iter()
      .filter(|(name, _)| !name.is_empty())
      .map(|(name, section)| read(&mut reader, name, section))
      .collect()
  }

  /// `environment.ltx`'s `[environment]`; on OpenXRay without the file, `system.ltx`'s `[thunderbolt_common]`. The
  /// engine builds its thunder whether or not a keyframe strikes, so it needs one or the other either way.
  pub(super) fn read_thunderbolt_settings(&mut self) -> XrfResult<Option<ThunderboltSettings>> {
    let path: XrayLogicalPath = self.project.config_path(Self::ENVIRONMENT)?;

    let (config, section): (Arc<EnvironmentConfig>, &str) = if self.exists(&path)? {
      match self.open(&path)? {
        Some(config) => (Arc::new(config), Self::THUNDERBOLT_SETTINGS),
        None => return Ok(None),
      }
    } else if self.options.engine == XrayEngine::Vanilla {
      match self.get_system()? {
        Some(system) => (system, Self::THUNDERBOLT_COMMON),
        None => return Ok(None),
      }
    } else {
      self.findings.push(EnvironmentFinding {
        file: path.as_str().to_owned(),
        key: None,
        message: format!(
          "There is no {}, where the engine reads where thunderbolts strike from",
          Self::ENVIRONMENT
        ),
        rule: EnvironmentRule::Engine,
        section: None,
      });

      return Ok(None);
    };

    let mut reader: EnvironmentSectionReader = self.reader("Thunderbolt settings", &config);

    match config.get_ltx().section(section) {
      Some(found) => Ok(Some(ThunderboltSettingsKey::read(&mut reader, section, found))),
      None => {
        reader.report_file(
          EnvironmentRule::Engine,
          format!("There is no [{section}] section, where the engine reads where thunderbolts strike from"),
        );

        Ok(None)
      }
    }
  }

  /// Each `environment\ambients\<level>.ltx`.
  ///
  /// A file `ambients.ltx` includes is left out: Anomaly keeps shared ambients beside the levels' own, and a file named
  /// for no level is never read as one.
  pub(super) fn read_level_ambients(&mut self) -> XrfResult<Vec<LevelAmbients>> {
    let shared: XrayLogicalPath = self.project.config_path(Self::AMBIENTS)?;
    let included: HashSet<String> = match self.open(&shared)? {
      Some(config) => config
        .get_ltx()
        .iter()
        .filter_map(|(_, section)| section.get_origin())
        .filter(|origin| *origin != shared.as_str())
        .map(str::to_owned)
        .collect(),
      None => HashSet::new(),
    };
    let mut levels: Vec<LevelAmbients> = Vec::new();

    for file in self.list_configs(Self::LEVEL_AMBIENTS)? {
      self.options.job.check_cancelled()?;

      if included.contains(file.as_str()) {
        continue;
      }

      let Some(config) = self.open(&file)? else {
        continue;
      };

      levels.push(LevelAmbients {
        ambients: self.read_sections(&config, "Ambient", AmbientKey::read),
        file: config.file.clone(),
        level: Self::stem_of(&file).to_owned(),
      });
    }

    Ok(levels)
  }

  /// Monolith's sun table, which it requires.
  pub(super) fn read_sun_table(&mut self) -> XrfResult<Option<SunTable>> {
    let path: XrayLogicalPath = self.project.config_path(Self::SUN_POSITIONS)?;

    if !self.exists(&path)? {
      self.findings.push(EnvironmentFinding {
        file: path.as_str().to_owned(),
        key: None,
        message: format!(
          "There is no {}, which the engine stands the sun by",
          Self::SUN_POSITIONS
        ),
        rule: EnvironmentRule::Engine,
        section: None,
      });

      return Ok(None);
    }

    let Some(config) = self.open(&path)? else {
      return Ok(None);
    };
    let mut reader: EnvironmentSectionReader = self.reader("Sun position", &config);

    Ok(Some(SunTable::read(&mut reader, config.get_ltx())))
  }

  pub(super) fn read_graphs(&mut self) -> XrfResult<WeatherGraphs> {
    let path: XrayLogicalPath = self.project.config_path(Self::GRAPHS)?;

    Ok(match self.open(&path)? {
      Some(config) => WeatherGraphs::read(config.get_ltx(), self.options.engine),
      None => WeatherGraphs::default(),
    })
  }

  /// A Shadow of Chernobyl ambient is its own sound channel, read from its own section.
  pub(super) fn read_own_channels(&mut self, ambients: &[Ambient]) -> XrfResult<Vec<SoundChannel>> {
    let own: Vec<&Ambient> = ambients
      .iter()
      .filter(|ambient| AmbientKey::is_own_channel(ambient, self.options.engine))
      .collect();

    if own.is_empty() {
      return Ok(Vec::new());
    }

    let path: XrayLogicalPath = self.project.config_path(Self::AMBIENTS)?;
    let Some(config) = self.open(&path)? else {
      return Ok(Vec::new());
    };
    let mut reader: EnvironmentSectionReader = self.reader("Ambient", &config);

    Ok(
      own
        .into_iter()
        .filter_map(|ambient| {
          config
            .get_ltx()
            .section(&ambient.name)
            .map(|section| (&ambient.name, section))
        })
        .map(|(name, section)| {
          let channel: SoundChannel = reader.read::<SoundChannelKey>(name, section);

          SoundChannelKey::judge(&mut reader, &channel);

          channel
        })
        .collect(),
    )
  }
}
