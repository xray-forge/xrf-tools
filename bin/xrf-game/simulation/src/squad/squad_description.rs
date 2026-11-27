use xrf_ltx::{Condlist, Section};

use crate::config::config_text::ConfigText;
use crate::squad::squad_size::SquadSize;

/// A squad section of `system.ltx` (`squad_descr*.ltx`), what `sim_squad_scripted:init_squad` and `create_npc` read:
/// whose it is, who it is made of, where it spawns and where scripts send it.
#[derive(Clone, Debug, PartialEq)]
pub struct SquadDescription {
  pub section: String,
  /// Its faction, the squad's `player_id` (`faction`).
  pub faction: String,
  /// Members it always has, one of each (`npc`).
  pub members: Vec<String>,
  /// Members it draws from, a name listed twice drawn twice as often (`npc_random`), and how many (`npc_in_squad`).
  pub random_members: Vec<String>,
  pub size: SquadSize,
  /// Where its members spawn: `self`, the smart terrain it is made at, or a patrol's first point (`spawn_point`).
  pub spawn_point: Condlist,
  /// Where scripts send it rather than the simulation (`target_smart`), none for a squad the simulation steers.
  pub target: Option<Condlist>,
  /// The behaviour section it weighs targets by, its faction's where it names none (`behaviour`).
  pub behaviour: String,
  /// Whether it walks even where it could run, and whether it counts as arrived wherever it stands (`always_walk`,
  /// `always_arrived`).
  pub always_walk: bool,
  pub always_arrived: bool,
}

impl SquadDescription {
  /// The squad `section` describes, or why it cannot be read: no faction, no member, or a condlist that does not parse.
  pub fn read(name: &str, section: &Section) -> Result<Self, String> {
    let faction: String = section
      .get("faction")
      .map(str::trim)
      .filter(|faction| !faction.is_empty())
      .ok_or_else(|| String::from("names no faction"))?
      .to_string();
    let members: Vec<String> = section.get("npc").map(ConfigText::split_names).unwrap_or_default();
    let random_members: Vec<String> = section
      .get("npc_random")
      .map(ConfigText::split_names)
      .unwrap_or_default();

    if members.is_empty() && random_members.is_empty() {
      return Err(String::from("names no member, neither npc nor npc_random"));
    }

    Ok(Self {
      section: String::from(name),
      behaviour: section
        .get("behaviour")
        .map(str::trim)
        .filter(|behaviour| !behaviour.is_empty())
        .map_or_else(|| faction.clone(), String::from),
      faction,
      members,
      random_members,
      size: Self::read_size(section)?,
      spawn_point: ConfigText::parse_condlist(section.get("spawn_point").unwrap_or("self"))?,
      target: section
        .get("target_smart")
        .filter(|target| !target.trim().is_empty())
        .map(ConfigText::parse_condlist)
        .transpose()?,
      always_walk: section.get_bool("always_walk").unwrap_or(false),
      always_arrived: section.get_bool("always_arrived").unwrap_or(false),
    })
  }

  /// `npc_in_squad`, two numbers as `utils.r_2nums` reads them, the default where it is missing; a minimum past the
  /// maximum is refused, as `create_npc` aborts on it.
  fn read_size(section: &Section) -> Result<SquadSize, String> {
    let Some(value) = section.get("npc_in_squad") else {
      return Ok(SquadSize::default());
    };
    let numbers: Vec<u32> = ConfigText::split_names(value)
      .iter()
      .map(|number| number.parse::<f32>().map(|number| number.max(0.0) as u32))
      .collect::<Result<_, _>>()
      .map_err(|_| format!("npc_in_squad '{value}' is no pair of numbers"))?;
    let size: SquadSize = match numbers.as_slice() {
      [min] => SquadSize { min: *min, max: *min },
      [min, max, ..] => SquadSize { min: *min, max: *max },
      [] => SquadSize::default(),
    };

    if size.min > size.max {
      return Err(format!("npc_in_squad '{value}' draws more at least than at most"));
    }

    Ok(size)
  }
}
