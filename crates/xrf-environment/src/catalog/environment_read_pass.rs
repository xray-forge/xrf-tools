use std::collections::HashSet;
use std::sync::Arc;

use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_job::JobScope;
use xrf_ltx::{LtxProject, LtxResolution, Section};
use xrf_vfs::XrayLogicalPath;

use crate::ambient::{
  Ambient, AmbientEffect, AmbientEffectKey, AmbientKey, LevelAmbients, SoundChannel, SoundChannelKey,
};
use crate::catalog::environment_catalog::EnvironmentCatalog;
use crate::catalog::environment_config::EnvironmentConfig;
use crate::catalog::environment_definitions::EnvironmentDefinitions;
use crate::catalog::environment_named::EnvironmentNamed;
use crate::catalog::environment_read_options::EnvironmentReadOptions;
use crate::finding::{EnvironmentFinding, EnvironmentRule};
use crate::level::WeatherGraphs;
use crate::section::EnvironmentSectionReader;
use crate::sun::{LensFlare, LensFlareKey, SunTable};
use crate::thunderbolt::{
  Thunderbolt, ThunderboltCollection, ThunderboltKey, ThunderboltSettings, ThunderboltSettingsKey,
};
use crate::weather::{WeatherCycle, WeatherCycleKind, WeatherKey};

/// One read of a game's environment configs: what it has found so far, and `system.ltx` once it has been needed.
pub(crate) struct EnvironmentReadPass<'a> {
  project: &'a LtxProject,
  engine: XrayEngine,
  options: &'a EnvironmentReadOptions,
  findings: Vec<EnvironmentFinding>,
  configs: Vec<String>,
  /// One unit per config read, and where cancellation is asked for between them.
  progress: JobScope,
  /// `system.ltx`, resolved on first need: OpenXRay looks there for a sun or a thunderbolt collection its own
  /// configs lack, and for the strike settings without `environment.ltx`.
  system: Option<Option<Arc<EnvironmentConfig>>>,
}

impl<'a> EnvironmentReadPass<'a> {
  pub const WEATHERS: &'static str = "environment\\weathers";
  pub const WEATHER_EFFECTS: &'static str = "environment\\weather_effects";
  pub const SUNS: &'static str = "environment\\suns.ltx";
  pub const SUN_POSITIONS: &'static str = "environment\\sun_positions.ltx";
  pub const THUNDERBOLT_COLLECTIONS: &'static str = "environment\\thunderbolt_collections.ltx";
  pub const THUNDERBOLTS: &'static str = "environment\\thunderbolts.ltx";
  pub const ENVIRONMENT: &'static str = "environment\\environment.ltx";
  pub const AMBIENTS: &'static str = "environment\\ambients.ltx";
  pub const LEVEL_AMBIENTS: &'static str = "environment\\ambients";
  pub const SOUND_CHANNELS: &'static str = "environment\\sound_channels.ltx";
  pub const EFFECTS: &'static str = "environment\\effects.ltx";
  pub const GRAPHS: &'static str = "environment\\dynamic_weather_graphs.ltx";
  pub const SYSTEM: &'static str = "system.ltx";

  /// The section `environment.ltx` holds the strike settings in, and the one OpenXRay reads from `system.ltx` without it.
  const THUNDERBOLT_SETTINGS: &'static str = "environment";
  const THUNDERBOLT_COMMON: &'static str = "thunderbolt_common";

  pub fn new(project: &'a LtxProject, engine: XrayEngine, options: &'a EnvironmentReadOptions) -> Self {
    Self {
      configs: Vec::new(),
      engine,
      findings: Vec::new(),
      options,
      progress: options.job.enter("environment", None),
      project,
      system: None,
    }
  }

  pub fn run(mut self) -> XrfResult<EnvironmentCatalog> {
    let cycles: Vec<WeatherCycle> = self.read_cycles(WeatherCycleKind::Cycle)?;
    let effects: Vec<WeatherCycle> = self.read_cycles(WeatherCycleKind::Effect)?;
    let mut suns: Vec<LensFlare> = self.read_definitions(Self::SUNS, "Sun", LensFlareKey::read)?;
    let mut thunderbolt_collections: Vec<ThunderboltCollection> = self.read_definitions(
      Self::THUNDERBOLT_COLLECTIONS,
      "Thunderbolt collection",
      ThunderboltCollection::read,
    )?;
    let mut thunderbolts: Vec<Thunderbolt> =
      self.read_definitions(Self::THUNDERBOLTS, "Thunderbolt", |reader, name, section| {
        reader.read::<ThunderboltKey>(name, section)
      })?;
    let thunderbolt_settings: Option<ThunderboltSettings> = self.read_thunderbolt_settings()?;
    let ambients: Vec<Ambient> = self.read_definitions(Self::AMBIENTS, "Ambient", AmbientKey::read)?;
    let mut sound_channels: Vec<SoundChannel> =
      self.read_definitions(Self::SOUND_CHANNELS, "Sound channel", SoundChannelKey::read)?;
    let ambient_effects: Vec<AmbientEffect> = self.read_definitions(Self::EFFECTS, "Effect", AmbientEffectKey::read)?;
    let level_ambients: Vec<LevelAmbients> = self.read_level_ambients()?;
    let sun_table: Option<SunTable> = match self.engine {
      XrayEngine::Extended => self.read_sun_table()?,
      XrayEngine::Vanilla => None,
    };
    let graphs: WeatherGraphs = self.read_graphs()?;

    sound_channels.extend(self.read_own_channels(&ambients)?);

    for cycle in cycles.iter().chain(&effects) {
      self.resolve_keyframe_references(cycle, &mut suns, &mut thunderbolt_collections, &ambients)?;
    }

    self.resolve_collection_references(&thunderbolt_collections, &mut thunderbolts)?;

    let level_ambient_lists = level_ambients.iter().flat_map(|level| level.ambients.iter());

    for ambient in ambients.iter().chain(level_ambient_lists) {
      self.resolve_ambient_references(ambient, &sound_channels, &ambient_effects);
    }

    let definitions: EnvironmentDefinitions = EnvironmentDefinitions {
      ambient_effects: &ambient_effects,
      ambients: &ambients,
      cycles: cycles.iter().chain(&effects).collect(),
      level_ambients: &level_ambients,
      sound_channels: &sound_channels,
      suns: &suns,
      thunderbolt_collections: &thunderbolt_collections,
      thunderbolts: &thunderbolts,
    };

    definitions.demote_unloaded(self.engine, &mut self.findings);

    let mut catalog: EnvironmentCatalog = EnvironmentCatalog {
      ambient_effects,
      ambients,
      configs: self.configs,
      cycles,
      effects,
      engine: self.engine,
      findings: self.findings,
      graphs,
      level_ambients,
      sound_channels,
      sun_table,
      suns,
      thunderbolt_collections,
      thunderbolt_settings,
      thunderbolts,
    };

    Self::sort(&mut catalog);

    Ok(catalog)
  }

  /// Every cycle or effect in its directory, a config the dialect attaches to another left out.
  fn read_cycles(&mut self, kind: WeatherCycleKind) -> XrfResult<Vec<WeatherCycle>> {
    let (directory, subject) = match kind {
      WeatherCycleKind::Cycle => (Self::WEATHERS, "Weather"),
      WeatherCycleKind::Effect => (Self::WEATHER_EFFECTS, "Weather effect"),
    };
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
      let name: &str = Self::stem_of(&file);
      let mut reader: EnvironmentSectionReader = EnvironmentSectionReader::new(
        self.engine,
        subject,
        &config.file,
        config.get_provenance(),
        &mut self.findings,
      );

      cycles.push(WeatherCycle::read(&mut reader, name, kind, config.get_ltx()));
    }

    Ok(cycles)
  }

  /// Every section of one definitions config, read by its kind's reader; none where the config is not there.
  fn read_definitions<T>(
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

    Ok(Self::read_sections(
      self.engine,
      &config,
      subject,
      &mut self.findings,
      read,
    ))
  }

  fn read_sections<T>(
    engine: XrayEngine,
    config: &EnvironmentConfig,
    subject: &'static str,
    findings: &mut Vec<EnvironmentFinding>,
    read: impl Fn(&mut EnvironmentSectionReader, &str, &Section) -> T,
  ) -> Vec<T> {
    let mut reader: EnvironmentSectionReader =
      EnvironmentSectionReader::new(engine, subject, &config.file, config.get_provenance(), findings);

    config
      .get_ltx()
      .iter()
      .filter(|(name, _)| !name.is_empty())
      .map(|(name, section)| read(&mut reader, name, section))
      .collect()
  }

  /// `environment.ltx`'s `[environment]`; on OpenXRay without the file, `system.ltx`'s `[thunderbolt_common]`. The
  /// engine builds its thunder whether or not a keyframe strikes, so it needs one or the other either way.
  fn read_thunderbolt_settings(&mut self) -> XrfResult<Option<ThunderboltSettings>> {
    let path: XrayLogicalPath = self.project.config_path(Self::ENVIRONMENT)?;

    let (config, section): (Arc<EnvironmentConfig>, &str) = if self.exists(&path)? {
      match self.open(&path)? {
        Some(config) => (Arc::new(config), Self::THUNDERBOLT_SETTINGS),
        None => return Ok(None),
      }
    } else if self.engine == XrayEngine::Vanilla {
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

    let mut reader: EnvironmentSectionReader = EnvironmentSectionReader::new(
      self.engine,
      "Thunderbolt settings",
      &config.file,
      config.get_provenance(),
      &mut self.findings,
    );

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
  fn read_level_ambients(&mut self) -> XrfResult<Vec<LevelAmbients>> {
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
        ambients: Self::read_sections(self.engine, &config, "Ambient", &mut self.findings, AmbientKey::read),
        file: config.file.clone(),
        level: Self::stem_of(&file).to_owned(),
      });
    }

    Ok(levels)
  }

  /// Monolith's sun table, which it requires.
  fn read_sun_table(&mut self) -> XrfResult<Option<SunTable>> {
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
    let mut reader: EnvironmentSectionReader = EnvironmentSectionReader::new(
      self.engine,
      "Sun position",
      &config.file,
      config.get_provenance(),
      &mut self.findings,
    );

    Ok(Some(SunTable::read(&mut reader, config.get_ltx())))
  }

  fn read_graphs(&mut self) -> XrfResult<WeatherGraphs> {
    let path: XrayLogicalPath = self.project.config_path(Self::GRAPHS)?;

    Ok(match self.open(&path)? {
      Some(config) => WeatherGraphs::read(config.get_ltx(), self.engine),
      None => WeatherGraphs::default(),
    })
  }

  /// A Shadow of Chernobyl ambient is its own sound channel, read from its own section.
  fn read_own_channels(&mut self, ambients: &[Ambient]) -> XrfResult<Vec<SoundChannel>> {
    let own: Vec<&Ambient> = ambients
      .iter()
      .filter(|ambient| AmbientKey::is_own_channel(ambient, self.engine))
      .collect();

    if own.is_empty() {
      return Ok(Vec::new());
    }

    let path: XrayLogicalPath = self.project.config_path(Self::AMBIENTS)?;
    let Some(config) = self.open(&path)? else {
      return Ok(Vec::new());
    };
    let mut reader: EnvironmentSectionReader = EnvironmentSectionReader::new(
      self.engine,
      "Ambient",
      &config.file,
      config.get_provenance(),
      &mut self.findings,
    );

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

  /// Checks the sun, collection and ambient a keyframe names, taking a sun or collection OpenXRay would find in
  /// `system.ltx` from there.
  fn resolve_keyframe_references(
    &mut self,
    cycle: &WeatherCycle,
    suns: &mut Vec<LensFlare>,
    collections: &mut Vec<ThunderboltCollection>,
    ambients: &[Ambient],
  ) -> XrfResult<()> {
    for keyframe in &cycle.keyframes {
      let section = &keyframe.section;
      let sun: &str = section.get_text(WeatherKey::Sun, self.engine);
      let collection: &str = section.get_text(WeatherKey::ThunderboltCollection, self.engine);
      let ambient: &str = section.get_text(WeatherKey::Ambient, self.engine);

      if !sun.is_empty() && !suns.iter().any(|it| it.name == sun) && !self.take_system_sun(sun, suns)? {
        self.report_reference(cycle, &section.name, "sun", format!("sun [{sun}]"));
      }

      if !collection.is_empty()
        && !collections.iter().any(|it| it.name == collection)
        && !self.take_system_collection(collection, collections)?
      {
        let target: String = format!("thunderbolt collection [{collection}]");

        self.report_reference(cycle, &section.name, "thunderbolt_collection", target);
      }

      if !ambient.is_empty() && !ambients.iter().any(|it| it.name == ambient) {
        self.report_reference(cycle, &section.name, "ambient", format!("ambient [{ambient}]"));
      }
    }

    Ok(())
  }

  /// Checks each collection's bolts: in `thunderbolts.ltx` for a collection of `thunderbolt_collections.ltx`, in
  /// `system.ltx` for one OpenXRay took from there.
  fn resolve_collection_references(
    &mut self,
    collections: &[ThunderboltCollection],
    thunderbolts: &mut Vec<Thunderbolt>,
  ) -> XrfResult<()> {
    let system: Option<String> = self
      .project
      .config_path(Self::SYSTEM)
      .ok()
      .map(|path| path.as_str().to_owned());

    for collection in collections {
      let is_system: bool = system.as_deref() == Some(collection.file.as_str());

      for bolt in &collection.thunderbolts {
        let is_found: bool = match is_system {
          true => self.take_system_section(bolt, "Thunderbolt", thunderbolts, |reader, name, section| {
            reader.read::<ThunderboltKey>(name, section)
          })?,
          false => thunderbolts.iter().any(|it| it.name == *bolt),
        };

        if !is_found {
          let message: String = format!(
            "Thunderbolt collection [{}] references missing thunderbolt [{bolt}]",
            collection.name
          );

          self.push_reference(&collection.file, &collection.name, None, message);
        }
      }
    }

    Ok(())
  }

  fn resolve_ambient_references(&mut self, ambient: &Ambient, channels: &[SoundChannel], effects: &[AmbientEffect]) {
    if !AmbientKey::is_own_channel(ambient, self.engine) {
      for channel in AmbientKey::list_channels(ambient) {
        if !channel.is_empty() && !channels.iter().any(|it| it.name == *channel) {
          let message: String = format!(
            "Ambient [{}] references missing sound channel [{channel}]",
            ambient.name
          );

          self.push_reference(&ambient.file, &ambient.name, Some("sound_channels"), message);
        }
      }
    }

    for effect in ambient.get_list(AmbientKey::Effects) {
      if !effect.is_empty() && !effects.iter().any(|it| it.name == *effect) {
        let message: String = format!("Ambient [{}] references missing effect [{effect}]", ambient.name);

        self.push_reference(&ambient.file, &ambient.name, Some("effects"), message);
      }
    }
  }

  /// On OpenXRay, a sun `system.ltx` holds in place of `suns.ltx`, added to the suns; whether one was found.
  fn take_system_sun(&mut self, name: &str, suns: &mut Vec<LensFlare>) -> XrfResult<bool> {
    self.take_system_section(name, "Sun", suns, LensFlareKey::read)
  }

  /// On OpenXRay, a collection `system.ltx` holds in place of `thunderbolt_collections.ltx`; whether one was found.
  fn take_system_collection(&mut self, name: &str, collections: &mut Vec<ThunderboltCollection>) -> XrfResult<bool> {
    self.take_system_section(name, "Thunderbolt collection", collections, ThunderboltCollection::read)
  }

  /// Reads one `system.ltx` section into a list, on OpenXRay only, unless it is there already.
  fn take_system_section<T>(
    &mut self,
    name: &str,
    subject: &'static str,
    list: &mut Vec<T>,
    read: impl Fn(&mut EnvironmentSectionReader, &str, &Section) -> T,
  ) -> XrfResult<bool>
  where
    T: EnvironmentNamed,
  {
    if self.engine != XrayEngine::Vanilla {
      return Ok(false);
    }

    let Some(system) = self.get_system()? else {
      return Ok(false);
    };

    if list
      .iter()
      .any(|it| it.get_name() == name && it.get_file() == system.file)
    {
      return Ok(true);
    }

    let Some(section) = system.get_ltx().section(name) else {
      return Ok(false);
    };
    let mut reader: EnvironmentSectionReader = EnvironmentSectionReader::new(
      self.engine,
      subject,
      &system.file,
      system.get_provenance(),
      &mut self.findings,
    );

    list.push(read(&mut reader, name, section));

    Ok(true)
  }

  fn report_reference(&mut self, cycle: &WeatherCycle, section: &str, key: &str, target: String) {
    let subject: &str = match cycle.kind {
      WeatherCycleKind::Cycle => "Weather",
      WeatherCycleKind::Effect => "Weather effect",
    };
    let message: String = format!("{subject} [{section}] references missing {target}");
    let file: &str = &cycle.file;

    self.push_reference(file, section, Some(key), message);
  }

  fn push_reference(&mut self, file: &str, section: &str, key: Option<&str>, message: String) {
    self.findings.push(EnvironmentFinding {
      file: file.to_owned(),
      key: key.map(str::to_owned),
      message,
      rule: EnvironmentRule::Reference,
      section: Some(section.to_owned()),
    });
  }

  /// `system.ltx`, resolved once.
  fn get_system(&mut self) -> XrfResult<Option<Arc<EnvironmentConfig>>> {
    if let Some(system) = &self.system {
      return Ok(system.clone());
    }

    let path: XrayLogicalPath = self.project.config_path(Self::SYSTEM)?;
    let system: Option<Arc<EnvironmentConfig>> = self.open(&path)?.map(Arc::new);

    self.system = Some(system.clone());

    Ok(system)
  }

  /// Whether a config is in the project's scope at all.
  fn exists(&self, path: &XrayLogicalPath) -> XrfResult<bool> {
    Ok(
      self
        .project
        .vfs()
        .scoped(self.project.scope())
        .find(path.as_str())?
        .is_some(),
    )
  }

  /// Resolves one config; none, and no finding, where it is not there, since each caller knows whether its absence is
  /// a problem, and none with a finding where it is there but will not resolve.
  fn open(&mut self, path: &XrayLogicalPath) -> XrfResult<Option<EnvironmentConfig>> {
    self.options.job.check_cancelled()?;

    if !self.exists(path)? {
      return Ok(None);
    }

    let resolved: XrfResult<Arc<LtxResolution>> = if self.options.is_explained {
      self.project.resolve_explained(path).map(Arc::new)
    } else {
      self.project.read_resolution(path)
    };

    self.progress.advance();
    self.configs.push(path.as_str().to_owned());

    match resolved {
      Ok(resolution) => Ok(Some(EnvironmentConfig {
        file: path.as_str().to_owned(),
        resolution,
      })),
      Err(error) => {
        self.findings.push(EnvironmentFinding {
          file: path.as_str().to_owned(),
          key: None,
          message: format!("Could not read the config: {error}"),
          rule: EnvironmentRule::Engine,
          section: None,
        });

        Ok(None)
      }
    }
  }

  /// The `.ltx` configs directly in a directory, a config the dialect attaches to another left out: under DLTX a
  /// `mod_` file patches its base rather than being a cycle of its own.
  fn list_configs(&self, relative: &str) -> XrfResult<Vec<XrayLogicalPath>> {
    let directory: XrayLogicalPath = self.project.config_path(relative)?;
    let files: Vec<XrayLogicalPath> = self
      .project
      .vfs()
      .scoped(self.project.scope())
      .list_children(directory.as_str())?
      .files
      .iter()
      .map(|asset| asset.get_logical_path().clone())
      .filter(|path| path.has_extension(XrayExtension::Ltx))
      .collect();
    let roots: Vec<String> = files.iter().map(|path| path.as_str().to_owned()).collect();
    let attachments: Vec<String> = self
      .project
      .get_dialect()
      .plan_attachments(&roots, &self.project.document_source())?;

    Ok(
      files
        .into_iter()
        .filter(|path| !attachments.iter().any(|attachment| attachment == path.as_str()))
        .collect(),
    )
  }

  /// A config's name without its extension, which is what the engine names a cycle or a level's ambients by.
  fn stem_of(path: &XrayLogicalPath) -> &str {
    let name: &str = path.file_name();

    name.rfind('.').map_or(name, |dot| &name[..dot])
  }

  fn sort(catalog: &mut EnvironmentCatalog) {
    catalog.cycles.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.effects.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.suns.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.thunderbolt_collections.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.thunderbolts.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.ambients.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.level_ambients.sort_by(|a, b| a.level.cmp(&b.level));
    catalog.sound_channels.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.ambient_effects.sort_by(|a, b| a.name.cmp(&b.name));
    catalog.configs.sort();
    catalog.configs.dedup();
    catalog.findings.sort();
    catalog.findings.dedup();
  }
}
