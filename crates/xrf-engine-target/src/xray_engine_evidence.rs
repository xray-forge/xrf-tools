use serde::{Deserialize, Serialize};

/// What decided a tree's engine.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayEngineEvidence {
  /// The caller named the engine.
  Named,
  /// `AnomalyLauncher.exe`, `bin/AnomalyDX*.exe` or `bin/VerifiedDX11.exe` beside `fsgame.ltx`.
  AnomalyExecutables,
  /// `fsgame.ltx` declares `$warfare_presets$`.
  AnomalyFsgame,
  /// `configs/environment/dynamic_weather_graphs.ltx` has a `[weather_cycles]` section.
  AtmosfearCycles,
  /// Nothing Anomaly or Atmosfear was found.
  NoSigns,
}

impl XrayEngineEvidence {
  /// What the evidence is, as a person reads it.
  pub const fn describe(self) -> &'static str {
    match self {
      Self::Named => "named",
      Self::AnomalyExecutables => "Anomaly executables found",
      Self::AnomalyFsgame => "Anomaly fsgame.ltx found",
      Self::AtmosfearCycles => "Atmosfear weather cycles found",
      Self::NoSigns => "no Anomaly or Atmosfear signs",
    }
  }
}
