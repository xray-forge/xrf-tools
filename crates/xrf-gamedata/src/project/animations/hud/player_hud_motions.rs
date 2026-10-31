use std::collections::BTreeSet;

use xrf_engine_target::XrayEngine;

/// The motions one player hands visual plays, asked the questions the engine asks when it binds a HUD's `anm_*` fields.
#[derive(Debug, Default)]
pub(crate) struct PlayerHudMotions {
  names: BTreeSet<String>,
}

impl PlayerHudMotions {
  /// How many numbered variants (`name1` to `name8`) the engine tries after the name itself.
  pub(crate) const VARIANTS: u32 = 8;

  /// The motion Monolith binds in place of a name the hands do not hold (`player_hud.cpp`, `player_hud_motion_container::load`).
  pub(crate) const IDLE_FALLBACK: &'static str = "hand_idle_doun";

  pub(crate) fn new(names: BTreeSet<String>) -> Self {
    Self { names }
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.names.is_empty()
  }

  /// Whether the hands hold `base` or one of its numbered variants, any of which the engine binds and picks among.
  pub(crate) fn plays(&self, base: &str) -> bool {
    self.names.contains(base) || (1..=Self::VARIANTS).any(|variant| self.names.contains(&format!("{base}{variant}")))
  }

  /// Whether `engine` binds the idle fallback for a name these hands do not play. Only Monolith has one: `xray-16`
  /// asserts instead.
  pub(crate) fn falls_back(&self, engine: XrayEngine) -> bool {
    match engine {
      XrayEngine::Vanilla => false,
      XrayEngine::Extended => self.names.contains(Self::IDLE_FALLBACK),
    }
  }
}
