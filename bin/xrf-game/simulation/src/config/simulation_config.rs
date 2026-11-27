use std::collections::{BTreeMap, BTreeSet};

use xrf_ltx::{Ltx, Section};

use crate::config::config_text::ConfigText;
use crate::config::simulation_config_source::SimulationConfigSource;
use crate::props::simulation_props::SimulationProps;
use crate::smart::smart_config::SmartConfig;
use crate::smart::smart_spawn::SmartSpawn;
use crate::squad::squad_behaviour::SquadBehaviour;
use crate::squad::squad_description::SquadDescription;
use crate::start::start_squad::StartSquad;

/// Everything the simulation reads from a tree's configs, once a session: the squads a new game starts with and the
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
  issues: Vec<String>,
}

impl SimulationConfig {
  /// The configs the scripts read beside `system.ltx`, by their path under `configs`.
  pub const START_FILE: &'static str = "misc\\simulation.ltx";
  pub const BEHAVIOURS_FILE: &'static str = "misc\\squad_behaviours.ltx";
  pub const PROPS_FILE: &'static str = "misc\\simulation_objects_props.ltx";

  /// Reads the simulation's configs from `source` for the graph's `levels` (in the graph's order) and the `smarts` the
  /// spawn places.
  pub fn read(source: &dyn SimulationConfigSource, levels: &[String], smarts: &[SmartSpawn]) -> Self {
    let mut config: Self = Self::default();

    config.behaviours = config.read_sections(source, Self::BEHAVIOURS_FILE, SquadBehaviour::read);
    config.props = config.read_sections(source, Self::PROPS_FILE, SimulationProps::read);

    for smart in smarts {
      match Self::read_smart(source, smart) {
        Ok(smart) => {
          config.smarts.insert(smart.name.clone(), smart);
        }
        Err(issue) => config.issues.push(format!("Smart terrain '{}' {issue}", smart.name)),
      }
    }

    match source.read_config(Self::START_FILE) {
      Ok(ltx) => config.start = config.read_start(&ltx, levels),
      Err(error) => config
        .issues
        .push(format!("{} does not read: {error}", Self::START_FILE)),
    }

    config.read_squads(source.get_system());
    config
  }

  pub fn get_squad(&self, section: &str) -> Option<&SquadDescription> {
    self.squads.get(section)
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

  /// What did not read, one line each.
  pub fn get_issues(&self) -> &[String] {
    &self.issues
  }

  /// Every section of the config at `path`, each read by `read`; a section that does not read is an issue.
  fn read_sections<T>(
    &mut self,
    source: &dyn SimulationConfigSource,
    path: &str,
    read: fn(&Section) -> Result<T, String>,
  ) -> BTreeMap<String, T> {
    let ltx: Ltx = match source.read_config(path) {
      Ok(ltx) => ltx,
      Err(error) => {
        self.issues.push(format!("{path} does not read: {error}"));

        return BTreeMap::new();
      }
    };

    ltx
      .iter()
      .filter_map(|(name, section)| match read(section) {
        Ok(value) => Some((String::from(name), value)),
        Err(issue) => {
          self.issues.push(format!("[{name}] of {path}: {issue}"));

          None
        }
      })
      .collect()
  }

  /// A smart terrain's config, from its custom data or the file its `cfg` names.
  fn read_smart(source: &dyn SimulationConfigSource, smart: &SmartSpawn) -> Result<SmartConfig, String> {
    let custom: Ltx =
      Ltx::read_from_str(&smart.custom_data).map_err(|error| format!("custom data does not read: {error}"))?;
    let file: Option<String> = custom
      .section(SmartConfig::SECTION)
      .and_then(|section| section.get("cfg"))
      .map(|cfg| cfg.trim().to_string())
      .filter(|cfg| !cfg.is_empty());
    let ltx: Ltx = match file {
      Some(file) => source
        .read_config(&file)
        .map_err(|error| format!("names cfg {file}, which does not read: {error}"))?,
      None => custom,
    };

    SmartConfig::read(&smart.name, &ltx)
  }

  /// The squads a new game starts with: vanilla's `[start_position_<level>]` sections, a level at a time in the graph's
  /// order, each line a squad and the smarts it is made at, in the engine's line order (`sim_board:fill_start_position`,
  /// which stopped at the first level without a section; here that level is skipped). A tree with none of them writes
  /// Anomaly's form instead: a section per smart, each line a squad and how many.
  fn read_start(&mut self, ltx: &Ltx, levels: &[String]) -> Vec<StartSquad> {
    let mut start: Vec<StartSquad> = Vec::new();
    let mut is_vanilla: bool = false;

    for level in levels {
      let Some(section) = ltx.section(&format!("start_position_{level}")) else {
        continue;
      };

      is_vanilla = true;

      for squad in ConfigText::sorted_keys(section) {
        for smart in ConfigText::split_names(section.get(squad).unwrap_or_default()) {
          self.push_start(&mut start, squad, smart);
        }
      }
    }

    if is_vanilla {
      return start;
    }

    let smarts: Vec<String> = self.smarts.keys().cloned().collect();

    for smart in smarts {
      let Some(section) = ltx.section(&smart) else {
        continue;
      };

      for squad in ConfigText::sorted_keys(section) {
        let count: usize = section
          .get(squad)
          .and_then(|count| count.trim().parse::<f32>().ok())
          .map_or(1, |count| count.max(0.0) as usize);

        for _ in 0..count {
          self.push_start(&mut start, squad, smart.clone());
        }
      }
    }

    start
  }

  /// Puts a start squad on the list, or an issue for a smart the spawn does not place (vanilla aborts on it).
  fn push_start(&mut self, start: &mut Vec<StartSquad>, squad: &str, smart: String) {
    if self.smarts.contains_key(&smart) {
      start.push(StartSquad {
        squad: String::from(squad),
        smart,
      });
    } else {
      self.issues.push(format!(
        "Start squad '{squad}' names smart terrain '{smart}', which has no config"
      ));
    }
  }

  /// Every squad section the start and the respawns name, read from `system`, with an issue for each it lacks or that
  /// names a member it lacks.
  fn read_squads(&mut self, system: &Ltx) {
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
      let Some(section) = system.section(&name) else {
        self.issues.push(format!("Squad section [{name}] is not in system.ltx"));
        continue;
      };

      match SquadDescription::read(&name, section) {
        Ok(squad) => {
          for member in squad.members.iter().chain(&squad.random_members) {
            if system.section(member).is_none() {
              self.issues.push(format!(
                "Squad [{name}] names member [{member}], which system.ltx lacks"
              ));
            }
          }

          self.squads.insert(name, squad);
        }
        Err(issue) => self.issues.push(format!("Squad [{name}] {issue}")),
      }
    }
  }
}
