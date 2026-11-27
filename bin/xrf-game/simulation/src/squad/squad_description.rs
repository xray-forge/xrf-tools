use xrf_game_config::{Condlist, ConfigReader, ConfigValue};

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
  /// The squad `section` describes, none where it cannot be made, said as an issue: no faction, no member, a size or
  /// a condlist that does not read.
  pub fn read(section: &mut ConfigReader<'_, '_>) -> Option<Self> {
    let faction: String = section.require("faction")?;
    let members: Vec<String> = section.read_names("npc");
    let random_members: Vec<String> = section.read_names("npc_random");

    if members.is_empty() && random_members.is_empty() {
      section.push_issue("names no member, neither npc nor npc_random");

      return None;
    }

    let size: SquadSize = Self::read_size(section)?;
    let spawn_point: Condlist = section.read_or_parse("spawn_point", "self")?;
    let target: Option<Condlist> = match section.get_text("target_smart") {
      Some(text) if !text.is_empty() => Some(section.read_or_parse("target_smart", text)?),
      _ => None,
    };

    Some(Self {
      section: String::from(section.get_name()),
      behaviour: section.read("behaviour").unwrap_or_else(|| faction.clone()),
      faction,
      members,
      random_members,
      size,
      spawn_point,
      target,
      always_walk: section.read_or("always_walk", false),
      always_arrived: section.read_or("always_arrived", false),
    })
  }

  /// `npc_in_squad`, two numbers as `utils.r_2nums` reads them, the default where it is missing; none, said as an
  /// issue, for one that does not read or a minimum past the maximum, as `create_npc` aborts on it.
  fn read_size(section: &mut ConfigReader<'_, '_>) -> Option<SquadSize> {
    let Some(text) = section.get_text("npc_in_squad") else {
      return Some(SquadSize::default());
    };
    let numbers: Vec<u32> = match ConfigReader::split_names(text)
      .iter()
      .map(|number| u32::parse(number))
      .collect::<Result<_, _>>()
    {
      Ok(numbers) => numbers,
      Err(reason) => {
        section.push_key("npc_in_squad", reason);

        return None;
      }
    };
    let size: SquadSize = match numbers.as_slice() {
      [min] => SquadSize { min: *min, max: *min },
      [min, max, ..] => SquadSize { min: *min, max: *max },
      [] => SquadSize::default(),
    };

    if size.min > size.max {
      section.push_key("npc_in_squad", format!("'{text}' draws more at least than at most"));

      return None;
    }

    Some(size)
  }
}
