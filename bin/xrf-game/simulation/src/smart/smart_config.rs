use xrf_game_config::{Condlist, ConfigFile, ConfigIssues, ConfigReader};

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

  /// The config of the smart `name`, read from `file` (its custom data, or the file its `cfg` names); none, said as
  /// an issue, where it cannot be: no `[smart_terrain]`, an unknown kind, or a respawn section or condlist that does
  /// not read.
  pub fn read(name: &str, file: &ConfigFile, issues: &mut ConfigIssues) -> Option<Self> {
    let mut section: ConfigReader<'_, '_> = file.require_section(Self::SECTION, issues)?;
    let kind: String = section.read_or("sim_type", String::from("default"));

    if !Self::KINDS.contains(&kind.as_str()) {
      section.push_key("sim_type", format!("'{kind}' is none of {}", Self::KINDS.join(", ")));

      return None;
    }

    let mut smart: Self = Self {
      name: String::from(name),
      kind,
      squad_id: section.read_or("squad_id", 0),
      max_population: section.read_or_parse("max_population", "0")?,
      respawns: Vec::new(),
      respawn_only: section.read_or("respawn_only_smart", false),
      respawn_radius: section.read_or("respawn_radius", Self::RESPAWN_RADIUS),
      respawn_idle: section.read_or("respawn_idle", Self::RESPAWN_IDLE),
      arrive_distance: section.read_or("arrive_dist", Self::ARRIVE_DISTANCE),
      mutant_lair: section.read_or("mutant_lair", false),
      no_mutant: section.read_or("no_mutant", false),
    };
    // An empty `respawn_params` is a smart that never respawns, as one that names none.
    let params: Option<&str> = section.get_text("respawn_params").filter(|params| !params.is_empty());

    if let Some(params) = params {
      smart.respawns = Self::read_respawns(file, params, issues)?;
    }

    Some(smart)
  }

  /// The respawn sections `params` lists, as `check_respawn_params` reads them: each line a section naming its squads
  /// and its count; none, said as an issue, where a section is missing or names no squad or count.
  fn read_respawns(file: &ConfigFile, params: &str, issues: &mut ConfigIssues) -> Option<Vec<SmartRespawn>> {
    let names: Vec<&str> = file.require_section(params, issues)?.get_keys();

    names
      .into_iter()
      .map(|name| {
        let mut section: ConfigReader<'_, '_> = file.require_section(name, issues)?;
        let squads: Vec<String> = section.read_names("spawn_squads");

        if squads.is_empty() {
          section.push_key("spawn_squads", "names no squad");

          return None;
        }

        Some(SmartRespawn {
          section: String::from(name),
          squads,
          count: section.require("spawn_num")?,
        })
      })
      .collect()
  }
}
