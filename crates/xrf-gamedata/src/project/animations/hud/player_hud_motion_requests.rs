use std::collections::{BTreeMap, BTreeSet};

use xrf_engine_target::XrayEngine;
use xrf_ltx::Ltx;

use crate::project::animations::hud::player_hud_motion_verdict::PlayerHudMotionVerdict;
use crate::project::animations::hud::player_hud_motions::PlayerHudMotions;
use crate::project::gamedata_item_section::GamedataItemSection;

/// The hands motions weapon HUD sections ask for, by motion: one missing motion is one problem however many HUD sections
/// and player hands it reaches.
#[derive(Debug, Default)]
pub(crate) struct PlayerHudMotionRequests {
  /// Motion name, then the HUD section asking for it, then the `anm_*` fields that do.
  requests: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
}

impl PlayerHudMotionRequests {
  /// The `anm_*` motions of every HUD section a weapon names. Each section is read once however many weapons share it.
  pub(crate) fn collect(system_ltx: &Ltx) -> Self {
    let mut requests: Self = Self::default();
    let huds: BTreeSet<&str> = system_ltx
      .iter()
      .filter(|(_, section)| GamedataItemSection::is_weapon(section))
      .filter_map(|(_, weapon)| weapon.get("hud"))
      .collect();

    // A weapon without a HUD section the hands can play is reported by the weapons check.
    for hud_name in huds {
      let Some(hud) = system_ltx.section(hud_name) else {
        continue;
      };

      for (field_name, field_value) in hud {
        if field_name.starts_with("anm_") {
          requests.add(hud_name, field_name, GamedataItemSection::get_motion_name(field_value));
        }
      }
    }

    requests
  }

  pub(crate) fn add(&mut self, hud: &str, field: &str, motion: &str) {
    self
      .requests
      .entry(motion.to_owned())
      .or_default()
      .entry(hud.to_owned())
      .or_default()
      .insert(field.to_owned());
  }

  /// Judges every requested motion against each player hands set the way `engine` binds it.
  pub(crate) fn verify(&self, hands: &[(String, PlayerHudMotions)], engine: XrayEngine) -> PlayerHudMotionVerdict {
    let mut verdict: PlayerHudMotionVerdict = PlayerHudMotionVerdict::default();

    for (motion, huds) in &self.requests {
      let missing: Vec<&(String, PlayerHudMotions)> =
        hands.iter().filter(|(_, motions)| !motions.plays(motion)).collect();

      if missing.is_empty() {
        continue;
      }

      verdict
        .invalid_hands
        .extend(missing.iter().map(|(name, _)| name.clone()));
      verdict.messages.push(format!(
        "Motion '{motion}', asked for by {}, is missing from {} under its own name and as '{motion}1' to '{motion}{}': {}",
        Self::describe_requests(huds),
        Self::describe_hands(&missing, hands.len()),
        PlayerHudMotions::VARIANTS,
        Self::describe_outcome(&missing, engine),
      ));
    }

    verdict
  }

  /// `[hud_a], [hud_b] as anm_x, anm_y and [hud_c] as anm_z`, sections asking through the same fields together.
  fn describe_requests(huds: &BTreeMap<String, BTreeSet<String>>) -> String {
    let mut by_fields: BTreeMap<&BTreeSet<String>, Vec<String>> = BTreeMap::new();

    for (hud, fields) in huds {
      by_fields.entry(fields).or_default().push(format!("[{hud}]"));
    }

    by_fields
      .into_iter()
      .map(|(fields, huds)| {
        format!(
          "{} as {}",
          huds.join(", "),
          fields.iter().map(String::as_str).collect::<Vec<&str>>().join(", ")
        )
      })
      .collect::<Vec<String>>()
      .join(" and ")
  }

  /// `all 101 player HUD sections`, or the ones named when not all of them miss it.
  fn describe_hands(missing: &[&(String, PlayerHudMotions)], hands_count: usize) -> String {
    if missing.len() == hands_count && hands_count > 1 {
      return format!("all {hands_count} player HUD sections");
    }

    let names: String = missing
      .iter()
      .map(|(name, _)| format!("[{name}]"))
      .collect::<Vec<String>>()
      .join(", ");

    match missing.len() {
      1 => format!("player HUD section {names}"),
      _ => format!("player HUD sections {names}"),
    }
  }

  /// What the engine does with hands that do not play the motion.
  fn describe_outcome(missing: &[&(String, PlayerHudMotions)], engine: XrayEngine) -> String {
    let without_fallback: Vec<String> = missing
      .iter()
      .filter(|(_, motions)| !motions.falls_back(engine))
      .map(|(name, _)| format!("[{name}]"))
      .collect();

    match engine {
      XrayEngine::Vanilla => String::from("the engine asserts on it when it binds the HUD to those hands"),
      XrayEngine::Extended if without_fallback.is_empty() => format!(
        "the extended engine binds '{}' in its place, so the motion asked for never plays",
        PlayerHudMotions::IDLE_FALLBACK
      ),
      XrayEngine::Extended => format!(
        "the extended engine binds '{}' in its place, but binds nothing in {}, where '{}' is missing too",
        PlayerHudMotions::IDLE_FALLBACK,
        without_fallback.join(", "),
        PlayerHudMotions::IDLE_FALLBACK
      ),
    }
  }
}
