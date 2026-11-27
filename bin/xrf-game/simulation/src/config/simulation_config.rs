use std::collections::{BTreeMap, BTreeSet};

use xrf_game_config::{
  CondlistCondition, ConfigFile, ConfigIssue, ConfigIssues, ConfigReader, ConfigSource, ConfigValue,
};
use xrf_spawn::ClsId;

use crate::props::simulation_props::SimulationProps;
use crate::smart::smart_config::SmartConfig;
use crate::smart::smart_spawn::SmartSpawn;
use crate::squad::squad_behaviour::SquadBehaviour;
use crate::squad::squad_description::SquadDescription;
use crate::start::start_squad::StartSquad;

/// Everything the simulation reads from a tree's configs, once a game start: the squads a new game starts with and the
/// smart terrains respawn, how each behaviour weighs targets, what each simulation object offers, and each smart
/// terrain's config. What does not read is kept as an issue and left out, never a failure: one broken squad does not
/// stop a game, as one broken script line does not stop X-Ray's.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SimulationConfig {
  squads: BTreeMap<String, SquadDescription>,
  behaviours: BTreeMap<String, SquadBehaviour>,
  props: BTreeMap<String, SimulationProps>,
  smarts: BTreeMap<String, SmartConfig>,
  start: Vec<StartSquad>,
  /// The class of every squad and member section the squads name, as `class` reads in `system.ltx`.
  classes: BTreeMap<String, ClsId>,
  issues: ConfigIssues,
}

impl SimulationConfig {
  /// The configs the scripts read beside `system.ltx`, by their path under `configs`.
  pub const START_FILE: &'static str = "misc\\simulation.ltx";
  pub const BEHAVIOURS_FILE: &'static str = "misc\\squad_behaviours.ltx";
  pub const PROPS_FILE: &'static str = "misc\\simulation_objects_props.ltx";

  /// What the issue of the condition functions the condlists call names as its place.
  const CONDLISTS: &'static str = "the simulation's condlists";

  /// Reads the simulation's configs from `source` for the graph's `levels` (in the graph's order) and the `smarts` the
  /// spawn places.
  pub fn read(source: &dyn ConfigSource, levels: &[String], smarts: &[SmartSpawn]) -> Self {
    let mut config: Self = Self::default();

    config.behaviours = config.read_sections(source, Self::BEHAVIOURS_FILE, |section| {
      Some(SquadBehaviour::read(section))
    });
    config.props = config.read_sections(source, Self::PROPS_FILE, SimulationProps::read);

    for smart in smarts {
      if let Some(read) = config.read_smart(source, smart) {
        config.smarts.insert(smart.name.clone(), read);
      }
    }

    match source.read_file(Self::START_FILE) {
      Ok(file) => config.start = config.read_start(&file, levels),
      Err(error) => config
        .issues
        .push_file(Self::START_FILE, format!("does not read: {error}")),
    }

    config.read_squads(source.get_system());
    config.report_functions();
    config
  }

  /// Adds one issue naming the condition functions its condlists call, which the game does not implement yet and
  /// which answer false unless a script registers them (`SimulationFunctions`), so a respawn count or a target that
  /// differs from X-Ray's is traced.
  fn report_functions(&mut self) {
    let condlists = self
      .smarts
      .values()
      .flat_map(|smart| {
        std::iter::once(&smart.max_population).chain(smart.respawns.iter().map(|respawn| &respawn.count))
      })
      .chain(
        self
          .squads
          .values()
          .flat_map(|squad| std::iter::once(&squad.spawn_point).chain(&squad.target)),
      )
      .chain(self.props.values().map(|props| &props.available));
    let functions: BTreeSet<&str> = condlists
      .flat_map(|condlist| &condlist.branches)
      .flat_map(|branch| &branch.conditions)
      .filter_map(|condition| match condition {
        CondlistCondition::Function { name, .. } => Some(name.as_str()),
        _ => None,
      })
      .collect();

    if !functions.is_empty() {
      let names: Vec<&str> = functions.into_iter().collect();
      let message: String = format!(
        "call {} condition functions the game does not implement yet, which answer false: {}",
        names.len(),
        names.join(", ")
      );

      self.issues.push_file(Self::CONDLISTS, message);
    }
  }

  pub fn get_squad(&self, section: &str) -> Option<&SquadDescription> {
    self.squads.get(section)
  }

  /// The class of a squad or member section the squads name.
  pub fn get_class(&self, section: &str) -> Option<&ClsId> {
    self.classes.get(section)
  }

  /// Every squad section a new game or a respawn names, by section.
  pub fn get_squads(&self) -> &BTreeMap<String, SquadDescription> {
    &self.squads
  }

  pub fn get_behaviour(&self, section: &str) -> Option<&SquadBehaviour> {
    self.behaviours.get(section)
  }

  /// How many behaviour sections it read.
  pub fn count_behaviours(&self) -> usize {
    self.behaviours.len()
  }

  pub fn get_props(&self, section: &str) -> Option<&SimulationProps> {
    self.props.get(section)
  }

  /// How many props sections it read.
  pub fn count_props(&self) -> usize {
    self.props.len()
  }

  pub fn get_smart(&self, name: &str) -> Option<&SmartConfig> {
    self.smarts.get(name)
  }

  /// Every smart terrain's config, by name.
  pub fn get_smarts(&self) -> &BTreeMap<String, SmartConfig> {
    &self.smarts
  }

  /// The squads a new game starts with, in the order it makes them.
  pub fn get_start(&self) -> &[StartSquad] {
    &self.start
  }

  /// What did not read.
  pub fn get_issues(&self) -> &ConfigIssues {
    &self.issues
  }

  /// Every section of the config at `path`, each read by `read`; one that does not read is an issue.
  fn read_sections<T>(
    &mut self,
    source: &dyn ConfigSource,
    path: &str,
    read: impl Fn(&mut ConfigReader<'_, '_>) -> Option<T>,
  ) -> BTreeMap<String, T> {
    let file: ConfigFile = match source.read_file(path) {
      Ok(file) => file,
      Err(error) => {
        self.issues.push_file(path, format!("does not read: {error}"));

        return BTreeMap::new();
      }
    };
    let names: Vec<&str> = file.get_section_names().collect();

    names
      .into_iter()
      .filter_map(|name| {
        let mut section: ConfigReader<'_, '_> = file.read_section(name, &mut self.issues)?;

        read(&mut section).map(|value| (String::from(name), value))
      })
      .collect()
  }

  /// A smart terrain's config, from its custom data or the file its `cfg` names; none, said as an issue, where it
  /// does not read.
  fn read_smart(&mut self, source: &dyn ConfigSource, smart: &SmartSpawn) -> Option<SmartConfig> {
    let origin: String = format!("custom data of smart terrain '{}'", smart.name);
    let custom: ConfigFile = match ConfigFile::parse(&origin, &smart.custom_data) {
      Ok(custom) => custom,
      Err(error) => {
        self.issues.push_file(&origin, format!("does not read: {error}"));

        return None;
      }
    };
    let cfg: Option<String> = custom
      .read_section(SmartConfig::SECTION, &mut self.issues)
      .and_then(|mut section| section.read("cfg"));
    let file: ConfigFile = match cfg {
      Some(cfg) => match source.read_file(&cfg) {
        Ok(file) => file,
        Err(error) => {
          self
            .issues
            .push_file(&origin, format!("names cfg {cfg}, which does not read: {error}"));

          return None;
        }
      },
      None => custom,
    };

    SmartConfig::read(&smart.name, &file, &mut self.issues)
  }

  /// The squads a new game starts with: vanilla's `[start_position_<level>]` sections, a level at a time in the graph's
  /// order, each line a squad and the smarts it is made at, in the engine's line order (`sim_board:fill_start_position`,
  /// which stopped at the first level without a section; here that level is skipped). A tree with none of them writes
  /// Anomaly's form instead: a section per smart, each line a squad and how many.
  fn read_start(&mut self, file: &ConfigFile, levels: &[String]) -> Vec<StartSquad> {
    let mut lines: Vec<StartSquad> = Vec::new();
    let mut is_vanilla: bool = false;

    for level in levels {
      let name: String = format!("start_position_{level}");
      let Some(section) = file.read_section(&name, &mut self.issues) else {
        continue;
      };

      is_vanilla = true;

      for squad in section.get_keys() {
        for smart in section.read_names(squad) {
          lines.push(StartSquad {
            squad: String::from(squad),
            smart,
          });
        }
      }
    }

    if !is_vanilla {
      let smarts: Vec<String> = self.smarts.keys().cloned().collect();

      for smart in &smarts {
        let Some(mut section) = file.read_section(smart, &mut self.issues) else {
          continue;
        };

        for squad in section.get_keys() {
          // A line with no count is one squad.
          let count: u32 = match section.get_text(squad) {
            Some(text) if !text.is_empty() => u32::parse(text).unwrap_or_else(|reason| {
              section.push_key(squad, format!("{reason}; one squad starts"));

              1
            }),
            _ => 1,
          };

          for _ in 0..count {
            lines.push(StartSquad {
              squad: String::from(squad),
              smart: smart.clone(),
            });
          }
        }
      }
    }

    lines.into_iter().filter(|line| self.has_smart(line)).collect()
  }

  /// Whether a start squad's smart has a config; an issue where it has none (vanilla aborts on it).
  fn has_smart(&mut self, line: &StartSquad) -> bool {
    if self.smarts.contains_key(&line.smart) {
      return true;
    }

    self.issues.push(ConfigIssue {
      file: String::from(Self::START_FILE),
      section: None,
      key: Some(line.squad.clone()),
      message: format!("names smart terrain '{}', which has no config", line.smart),
    });

    false
  }

  /// Every squad section the start and the respawns name, read from `system`, with an issue for each it lacks or that
  /// names a member it lacks.
  fn read_squads(&mut self, system: &ConfigFile) {
    let names: BTreeSet<String> = self
      .start
      .iter()
      .map(|start| start.squad.clone())
      .chain(
        self
          .smarts
          .values()
          .flat_map(|smart| smart.respawns.iter().flat_map(|respawn| respawn.squads.iter().cloned())),
      )
      .collect();

    for name in names {
      let Some(squad) = system
        .require_section(&name, &mut self.issues)
        .and_then(|mut section| SquadDescription::read(&mut section))
      else {
        continue;
      };

      if !self.read_class(system, &name) {
        continue;
      }

      for member in squad.members.iter().chain(&squad.random_members) {
        if !self.classes.contains_key(member) && !self.read_class(system, member) {
          self.issues.push(ConfigIssue {
            file: String::from(system.get_path()),
            section: Some(name.clone()),
            key: None,
            message: format!("names member [{member}], which has no class"),
          });
        }
      }

      self.squads.insert(name, squad);
    }
  }

  /// Reads the class of `section` (its `class`, as `F_entity_Create` builds an object of it), answering whether it has
  /// one the game knows; an issue where it does not.
  fn read_class(&mut self, system: &ConfigFile, section: &str) -> bool {
    let Some(mut reader) = system.require_section(section, &mut self.issues) else {
      return false;
    };
    let Some(class) = reader.require::<String>("class") else {
      return false;
    };

    match ClsId::from_class_name(&class) {
      Some(class) => {
        self.classes.insert(String::from(section), class);

        true
      }
      None => {
        reader.push_key("class", format!("{class} is a class the game does not know"));

        false
      }
    }
  }
}
