use std::collections::HashSet;
use std::path::PathBuf;

use xrf_collections::OrderedMap;
use xrf_job::JobHandle;
use xrf_ltx::{Condlist, CondlistCondition, Ltx, Section};
use xrf_spawn::{ClsId, GraphLevel, SpawnFile};
use xrf_vfs::XrayLogicalPath;

use crate::project::dialogs::squads::dialog_squad_member::DialogSquadMember;
use crate::project::gamedata_project::GamedataProject;

/// The squads a tree's configs spawn and the ones it only declares, and what the NPCs they place need to be created.
///
/// `sim_board:fill_start_position` creates the squads `misc\simulation.ltx` lists under `[start_position_<level>]` when
/// a new game starts, a smart terrain the squads its `spawn_squads` name as it respawns, and the `create_squad` effect
/// the squad its condlist names. A squad section places every NPC section its `npc` lists and one of those its
/// `npc_random` lists, and an NPC with a trading part is spawned with its section's `character_profile`, `default` where
/// it names none. `alife():create` asserts on a squad or NPC section no config declares (`F_entity_Create`).
///
/// Scripts create squads too, by names they pick from a table as they run (`pri_a28_check_zones` in vanilla and
/// xrf-engine alike), which a reader cannot follow. Every `ON_OFF_S` section is a squad something may create, so the
/// NPC sections of the ones the configs do not spawn are read as well, and kept apart.
pub(crate) struct DialogSquadSpawns {
  /// Each profile a spawned squad places an NPC with, by the first squad and NPC section placing it.
  pub(crate) profiles: OrderedMap<String, DialogSquadMember>,
  /// Each squad a config spawns that no section declares, by the reported path of the first config naming it.
  pub(crate) missing_squads: OrderedMap<String, PathBuf>,
  /// Each NPC section a spawned squad lists that no section declares, with the first squad listing it.
  pub(crate) missing_members: Vec<DialogSquadMember>,
  /// Each NPC section that only squads the configs do not spawn list and no section declares, with the first squad
  /// listing it.
  pub(crate) unspawned_missing_members: Vec<DialogSquadMember>,
}

impl DialogSquadSpawns {
  const SIMULATION_PATH: &'static str = "configs\\misc\\simulation.ltx";
  const START_POSITION_PREFIX: &'static str = "start_position_";
  const SPAWN_SQUADS_KEY: &'static str = "spawn_squads";
  const CREATE_SQUAD_EFFECT: &'static str = "create_squad";
  const MEMBER_KEYS: [&'static str; 2] = ["npc", "npc_random"];
  const CLASS_KEY: &'static str = "class";
  const PROFILE_KEY: &'static str = "character_profile";
  /// The profile `CSE_ALifeTraderAbstract` gives an NPC whose section names none.
  const DEFAULT_PROFILE: &'static str = "default";

  /// The squads the tree's configs spawn and the ones `system` declares, their sections looked up in `system`, which
  /// every squad section is part of.
  pub(crate) fn read(project: &GamedataProject, system: &Ltx, job: &JobHandle) -> Self {
    let mut spawned: OrderedMap<String, PathBuf> = Self::list_started(project);

    // A config that does not resolve is reported by the LTX check, not this one.
    for (path, outcome) in project.visit_configs(job, |ltx, _| Self::list_named(ltx)) {
      for squad in outcome.unwrap_or_default() {
        spawned.entry(squad).or_insert_with(|| path.clone());
      }
    }

    let mut spawns: Self = Self {
      profiles: OrderedMap::default(),
      missing_squads: OrderedMap::default(),
      missing_members: Vec::new(),
      unspawned_missing_members: Vec::new(),
    };
    let mut listed: HashSet<&str> = HashSet::new();

    for (squad, path) in &spawned {
      let Some(squad_section) = system.section(squad) else {
        spawns.missing_squads.insert(squad.clone(), path.clone());
        continue;
      };

      for npc in Self::list_members(squad_section).filter(|npc| listed.insert(npc)) {
        let member: DialogSquadMember = DialogSquadMember::new(squad, npc);

        match system.section(npc) {
          None => spawns.missing_members.push(member),
          Some(section) => {
            if let Some(profile) = Self::find_profile(section) {
              spawns.profiles.entry(profile).or_insert(member);
            }
          }
        }
      }
    }

    // Their profiles are judged with every section's `character_profile`; only a section no one declares is left.
    for (squad, squad_section) in system {
      if spawned.contains_key(squad) || !Self::is_squad(squad_section) {
        continue;
      }

      for npc in Self::list_members(squad_section).filter(|npc| listed.insert(npc)) {
        if system.section(npc).is_none() {
          spawns
            .unspawned_missing_members
            .push(DialogSquadMember::new(squad, npc));
        }
      }
    }

    spawns
  }

  /// The squads `misc\simulation.ltx` starts a new game with, by the file's reported path.
  ///
  /// `sim_board:fill_start_position` walks the levels of the game graph in id order and creates the keys of each one's
  /// `[start_position_<level>]`, returning at the first level with no such section; sections of levels the graph does
  /// not hold, which vanilla keeps for its test levels, are never read. A tree whose spawn files carry no game graph
  /// starts no game, so none of its start positions are read.
  fn list_started(project: &GamedataProject) -> OrderedMap<String, PathBuf> {
    let Ok(path) = XrayLogicalPath::new(Self::SIMULATION_PATH) else {
      return OrderedMap::default();
    };
    let Ok(simulation) = project.ltx_project.read_resolution(&path) else {
      return OrderedMap::default();
    };
    let reported: PathBuf = project.ltx_project.path_of(&path);
    let mut started: OrderedMap<String, PathBuf> = OrderedMap::default();

    for level in Self::list_graph_levels(project) {
      let Some(section) = simulation
        .ltx
        .section(&format!("{}{level}", Self::START_POSITION_PREFIX))
      else {
        break;
      };

      for (squad, _) in section {
        started.entry(squad.to_owned()).or_insert_with(|| reported.clone());
      }
    }

    started
  }

  /// The names of the levels the game graph holds, in id order: the graph of the first spawn file carrying one, which
  /// is the root spawn in a shipped tree. A spawn file that does not read is reported by the spawns check.
  fn list_graph_levels(project: &GamedataProject) -> Vec<String> {
    for spawn_path in project.spawn_paths() {
      let Ok(spawn) = project.read_parsed(&spawn_path, SpawnFile::read_from_chunk) else {
        continue;
      };
      let mut levels: Vec<&GraphLevel> = spawn.graphs.levels.iter().collect();

      if levels.is_empty() {
        continue;
      }

      levels.sort_by_key(|level| level.id);

      return levels.into_iter().map(|level| level.name.clone()).collect();
    }

    Vec::new()
  }

  /// The squads one config names to spawn: in a `spawn_squads` list, or as the first parameter of `create_squad`.
  fn list_named(ltx: &Ltx) -> Vec<String> {
    let mut named: Vec<String> = Vec::new();

    for (_, section) in ltx {
      for (key, value) in section {
        if key == Self::SPAWN_SQUADS_KEY {
          named.extend(Self::split_names(value).map(str::to_owned));
        } else if value.contains(Self::CREATE_SQUAD_EFFECT)
          && let Ok(condlist) = Condlist::parse(value)
        {
          named.extend(
            condlist
              .branches
              .iter()
              .flat_map(|branch| &branch.effects)
              .filter_map(|effect| match effect {
                CondlistCondition::Function {
                  name,
                  parameters: Some(parameters),
                  ..
                } if name == Self::CREATE_SQUAD_EFFECT => parameters.first().map(|squad| squad.trim().to_owned()),
                _ => None,
              }),
          );
        }
      }
    }

    named
  }

  /// The NPC sections a squad section lists, in its `npc` and then its `npc_random`.
  fn list_members(squad_section: &Section) -> impl Iterator<Item = &str> {
    Self::MEMBER_KEYS
      .iter()
      .filter_map(|key| squad_section.get(key))
      .flat_map(Self::split_names)
  }

  /// The names of a comma-separated list, trimmed, empty ones left out.
  fn split_names(value: &str) -> impl Iterator<Item = &str> {
    value.split(',').map(str::trim).filter(|name| !name.is_empty())
  }

  /// Whether a section is a squad's, which `sim_board:create_squad` can create.
  fn is_squad(section: &Section) -> bool {
    Self::find_class(section) == Some(ClsId::OnOffS)
  }

  /// The class a section's `class` names, `None` where it names none the engine knows.
  fn find_class(section: &Section) -> Option<ClsId> {
    section
      .get(Self::CLASS_KEY)
      .and_then(|class| ClsId::from_class_name(class.trim()))
  }

  /// The profile an NPC section spawns with, or `None` for a class without a trading part, which reads none.
  fn find_profile(section: &Section) -> Option<String> {
    matches!(Self::find_class(section)?, ClsId::AiStlS | ClsId::AiTrdS).then(|| {
      section
        .get(Self::PROFILE_KEY)
        .map(str::trim)
        .filter(|profile| !profile.is_empty())
        .unwrap_or(Self::DEFAULT_PROFILE)
        .to_owned()
    })
  }
}
