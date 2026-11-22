use indexmap::{IndexMap, IndexSet};
use xrf_job::JobHandle;
use xrf_ltx::{Condlist, CondlistCondition, Ltx, Section};
use xrf_spawn::ClsId;
use xrf_vfs::XrayLogicalPath;

use crate::project::dialogs::squads::dialog_squad_member::DialogSquadMember;
use crate::project::gamedata_project::GamedataProject;

/// The squads a tree's configs spawn, and the profile each NPC they place is spawned with.
///
/// `sim_board:fill_start_position` creates the squads `misc\simulation.ltx` lists under `[start_position_<level>]` when
/// a new game starts, a smart terrain the squads its `spawn_squads` name as it respawns, and the `create_squad` effect
/// the squad its condlist names. A squad section places every NPC section its `npc` lists and one of those its
/// `npc_random` lists, and an NPC with a trading part is spawned with its section's `character_profile`, `default` where
/// it names none.
pub(crate) struct DialogSquadSpawns {
  /// Each profile a spawned squad places an NPC with, by the first squad and NPC section placing it.
  pub(crate) profiles: IndexMap<String, DialogSquadMember>,
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

  /// The squads the tree's configs spawn, their sections looked up in `system`, which every squad section is part of.
  pub(crate) fn read(project: &GamedataProject, system: &Ltx, job: &JobHandle) -> Self {
    // todo: Read the squads scripts create by a literal name through `xr_effects.create_squad` or `sim_board:create_squad`.
    let mut squads: IndexSet<String> = Self::list_started(project);

    // A config that does not resolve is reported by the LTX check, not this one.
    for (_, outcome) in project.visit_configs(job, |ltx, _| Self::list_named(ltx)) {
      squads.extend(outcome.unwrap_or_default());
    }

    let mut profiles: IndexMap<String, DialogSquadMember> = IndexMap::new();

    // A squad no section declares stops `alife():create` before any NPC is placed, which is no profile's defect.
    for squad in squads {
      let Some(squad_section) = system.section(&squad) else {
        continue;
      };

      for npc in Self::MEMBER_KEYS
        .iter()
        .filter_map(|key| squad_section.get(key))
        .flat_map(|names| names.split(','))
        .map(str::trim)
        .filter(|name| !name.is_empty())
      {
        if let Some(profile) = system.section(npc).and_then(Self::find_profile) {
          profiles.entry(profile).or_insert_with(|| DialogSquadMember {
            squad: squad.clone(),
            section: npc.to_owned(),
          });
        }
      }
    }

    Self { profiles }
  }

  /// The squads `misc\simulation.ltx` starts a new game with, its keys under each `[start_position_<level>]`.
  fn list_started(project: &GamedataProject) -> IndexSet<String> {
    let Ok(path) = XrayLogicalPath::new(Self::SIMULATION_PATH) else {
      return IndexSet::new();
    };
    let Ok(simulation) = project.ltx_project.read_resolution(&path) else {
      return IndexSet::new();
    };

    (&simulation.ltx)
      .into_iter()
      .filter(|(name, _)| name.starts_with(Self::START_POSITION_PREFIX))
      .flat_map(|(_, section)| section.iter().map(|(key, _)| key.to_owned()))
      .collect()
  }

  /// The squads one config names to spawn: in a `spawn_squads` list, or as the first parameter of `create_squad`.
  fn list_named(ltx: &Ltx) -> Vec<String> {
    let mut named: Vec<String> = Vec::new();

    for (_, section) in ltx {
      for (key, value) in section.iter() {
        if key == Self::SPAWN_SQUADS_KEY {
          named.extend(
            value
              .split(',')
              .map(str::trim)
              .filter(|name| !name.is_empty())
              .map(str::to_owned),
          );
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
                } if name == Self::CREATE_SQUAD_EFFECT => parameters.first().cloned(),
                _ => None,
              }),
          );
        }
      }
    }

    named
  }

  /// The profile an NPC section spawns with, or `None` for a class without a trading part, which reads none.
  fn find_profile(section: &Section) -> Option<String> {
    let class: ClsId = section
      .get(Self::CLASS_KEY)
      .and_then(|class| ClsId::from_class_name(class.trim()))?;

    matches!(class, ClsId::AiStlS | ClsId::AiTrdS).then(|| {
      section
        .get(Self::PROFILE_KEY)
        .map(str::trim)
        .filter(|profile| !profile.is_empty())
        .unwrap_or(Self::DEFAULT_PROFILE)
        .to_owned()
    })
  }
}
