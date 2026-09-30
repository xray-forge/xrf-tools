use std::sync::Arc;

use xrf_engine_target::XrayEngine;
use xrf_error::XrfResult;
use xrf_extension::XrayExtension;
use xrf_job::JobScope;
use xrf_ltx::{LtxProject, LtxResolution};
use xrf_vfs::XrayLogicalPath;

use crate::ambient::{
  Ambient, AmbientEffect, AmbientEffectKey, AmbientKey, LevelAmbients, SoundChannel, SoundChannelKey,
};
use crate::catalog::environment_catalog::EnvironmentCatalog;
use crate::catalog::environment_config::EnvironmentConfig;
use crate::catalog::environment_definitions::EnvironmentDefinitions;
use crate::catalog::environment_read_options::EnvironmentReadOptions;
use crate::finding::{EnvironmentFinding, EnvironmentRule};
use crate::level::WeatherGraphs;
use crate::section::EnvironmentSectionReader;
use crate::sun::{LensFlare, LensFlareKey, SunTable};
use crate::thunderbolt::{Thunderbolt, ThunderboltCollection, ThunderboltKey, ThunderboltSettings};
use crate::weather::{WeatherCycle, WeatherCycleId, WeatherCycleKind};

/// One read of a game's environment configs: what it has found so far, and `system.ltx` once it has been needed.
pub(crate) struct EnvironmentReadPass<'a> {
  pub(super) project: &'a LtxProject,
  pub(super) options: &'a EnvironmentReadOptions,
  pub(super) findings: Vec<EnvironmentFinding>,
  pub(super) configs: Vec<String>,
  /// One unit per config read, and where cancellation is asked for between them.
  pub(super) progress: JobScope,
  /// `system.ltx`, resolved on first need: OpenXRay looks there for a sun or a thunderbolt collection its own
  /// configs lack, and for the strike settings without `environment.ltx`.
  pub(super) system: Option<Option<Arc<EnvironmentConfig>>>,
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
  pub(super) const THUNDERBOLT_SETTINGS: &'static str = "environment";
  pub(super) const THUNDERBOLT_COMMON: &'static str = "thunderbolt_common";

  pub fn new(project: &'a LtxProject, options: &'a EnvironmentReadOptions) -> Self {
    Self {
      configs: Vec::new(),
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
    let sun_table: Option<SunTable> = match self.options.engine {
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

    definitions.demote_unloaded(self.options.engine, &mut self.findings);

    let mut catalog: EnvironmentCatalog = EnvironmentCatalog {
      ambient_effects,
      ambients,
      configs: self.configs,
      cycles,
      effects,
      engine: self.options.engine,
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

  /// One cycle or effect by name, and what its own config says about it; none where there is no such config.
  pub fn run_cycle(mut self, id: &WeatherCycleId) -> XrfResult<Option<(WeatherCycle, Vec<EnvironmentFinding>)>> {
    let path: XrayLogicalPath = self
      .project
      .config_path(Self::directory_of(id.kind))?
      .join(&format!("{}.ltx", id.name))?;
    let Some(config) = self.open(&path)? else {
      return Ok(None);
    };
    let mut reader: EnvironmentSectionReader = self.reader(id.kind.get_subject(), &config);
    let cycle: WeatherCycle = WeatherCycle::read(&mut reader, id, config.get_ltx());

    self.findings.sort();

    Ok(Some((cycle, self.findings)))
  }

  /// Where a kind of cycle is kept.
  pub(super) fn directory_of(kind: WeatherCycleKind) -> &'static str {
    match kind {
      WeatherCycleKind::Cycle => Self::WEATHERS,
      WeatherCycleKind::Effect => Self::WEATHER_EFFECTS,
    }
  }

  /// A reader of one config's sections, reporting into this read's findings.
  pub(super) fn reader<'b>(
    &'b mut self,
    subject: &'static str,
    config: &'b EnvironmentConfig,
  ) -> EnvironmentSectionReader<'b> {
    EnvironmentSectionReader {
      engine: self.options.engine,
      file: &config.file,
      findings: &mut self.findings,
      provenance: config.get_provenance(),
      subject,
    }
  }

  /// `system.ltx`, resolved once.
  pub(super) fn get_system(&mut self) -> XrfResult<Option<Arc<EnvironmentConfig>>> {
    if let Some(system) = &self.system {
      return Ok(system.clone());
    }

    let path: XrayLogicalPath = self.project.config_path(Self::SYSTEM)?;
    let system: Option<Arc<EnvironmentConfig>> = self.open(&path)?.map(Arc::new);

    self.system = Some(system.clone());

    Ok(system)
  }

  /// Whether a config is in the project's scope at all.
  pub(super) fn exists(&self, path: &XrayLogicalPath) -> XrfResult<bool> {
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
  pub(super) fn open(&mut self, path: &XrayLogicalPath) -> XrfResult<Option<EnvironmentConfig>> {
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
  pub(super) fn list_configs(&self, relative: &str) -> XrfResult<Vec<XrayLogicalPath>> {
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
  pub(super) fn stem_of(path: &XrayLogicalPath) -> &str {
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
