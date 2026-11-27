use xrf_ltx::{Condlist, Ltx, Section};

use crate::config::config_text::ConfigText;
use crate::smart::smart_respawn::SmartRespawn;

/// A smart terrain's config, its `[smart_terrain]` section as `se_smart_terrain:read_params` reads it (from the custom
/// data, or the `cfg` file it names): its kind, how many squads it holds and how it respawns them.
#[derive(Clone, Debug, PartialEq)]
pub struct SmartConfig {
  pub name: String,
  /// Its simulation kind (`sim_type`: `default`, `base`, `resource`, `territory`).
  pub kind: String,
  /// The squad number its members take (`squad_id`).
  pub squad_id: u32,
  /// How many squads it holds at most (`max_population`, a condlist picking the number; 0 where it names none, so a
  /// smart without it never respawns).
  pub max_population: Condlist,
  /// How it respawns (`respawn_params`), its sections in the engine's order; none for a smart that never does.
  pub respawns: Vec<SmartRespawn>,
  /// Whether it only respawns and is never a squad's target (`respawn_only_smart`).
  pub respawn_only: bool,
  /// How far from it the actor must stand for it to respawn (`respawn_radius`, 150 m; vanilla read it and kept a
  /// constant 150 m instead).
  pub respawn_radius: f32,
  /// Game seconds between its respawn tries (`respawn_idle`; vanilla's constant 1,000 where it names none).
  pub respawn_idle: f32,
  /// How near a squad must come to have arrived (`arrive_dist`, 30 m).
  pub arrive_distance: f32,
  /// Whether it is a monster lair, and whether monsters keep away (`mutant_lair`, `no_mutant`).
  pub mutant_lair: bool,
  pub no_mutant: bool,
}

impl SmartConfig {
  /// The section custom data configures a smart terrain under.
  pub const SECTION: &'static str = "smart_terrain";

  /// The kinds `sim_type` may name (`smart_terrain.valid_territory`).
  pub const KINDS: [&'static str; 4] = ["default", "base", "resource", "territory"];

  /// Vanilla's `respawn_radius`, `RESPAWN_IDLE` and `arrive_dist` defaults.
  const RESPAWN_RADIUS: f32 = 150.0;
  const RESPAWN_IDLE: f32 = 1_000.0;
  const ARRIVE_DISTANCE: f32 = 30.0;

  /// The config of the smart `name`, read from `ltx` (its custom data, or the file its `cfg` names), or why it cannot
  /// be: no `[smart_terrain]`, an unknown kind, or a respawn section or condlist that does not read.
  pub fn read(name: &str, ltx: &Ltx) -> Result<Self, String> {
    let section: &Section = ltx
      .section(Self::SECTION)
      .ok_or_else(|| format!("holds no [{}]", Self::SECTION))?;
    let kind: String = section.get("sim_type").map_or("default", str::trim).to_string();

    if !Self::KINDS.contains(&kind.as_str()) {
      return Err(format!("sim_type '{kind}' is none of {}", Self::KINDS.join(", ")));
    }

    Ok(Self {
      name: String::from(name),
      kind,
      squad_id: section.get_f32("squad_id").map_or(0, |id| id.max(0.0) as u32),
      max_population: ConfigText::parse_condlist(section.get("max_population").unwrap_or("0"))?,
      respawns: match section.get("respawn_params").map(str::trim) {
        Some(params) if !params.is_empty() => Self::read_respawns(ltx, params)?,
        _ => Vec::new(),
      },
      respawn_only: section.get_bool("respawn_only_smart").unwrap_or(false),
      respawn_radius: section.get_f32("respawn_radius").unwrap_or(Self::RESPAWN_RADIUS),
      respawn_idle: section.get_f32("respawn_idle").unwrap_or(Self::RESPAWN_IDLE),
      arrive_distance: section.get_f32("arrive_dist").unwrap_or(Self::ARRIVE_DISTANCE),
      mutant_lair: section.get_bool("mutant_lair").unwrap_or(false),
      no_mutant: section.get_bool("no_mutant").unwrap_or(false),
    })
  }

  /// The respawn sections `params` lists, as `check_respawn_params` reads them: each line a section naming its squads
  /// and its count.
  fn read_respawns(ltx: &Ltx, params: &str) -> Result<Vec<SmartRespawn>, String> {
    let list: &Section = ltx
      .section(params)
      .ok_or_else(|| format!("respawn_params names [{params}], which it lacks"))?;

    ConfigText::sorted_keys(list)
      .into_iter()
      .map(|name| {
        let section: &Section = ltx
          .section(name)
          .ok_or_else(|| format!("[{params}] lists [{name}], which it lacks"))?;
        let squads: Vec<String> = section
          .get("spawn_squads")
          .map(ConfigText::split_names)
          .unwrap_or_default();

        if squads.is_empty() {
          return Err(format!("[{name}] names no spawn_squads"));
        }

        Ok(SmartRespawn {
          section: String::from(name),
          squads,
          count: ConfigText::parse_condlist(
            section
              .get("spawn_num")
              .ok_or_else(|| format!("[{name}] names no spawn_num"))?,
          )?,
        })
      })
      .collect()
  }
}
