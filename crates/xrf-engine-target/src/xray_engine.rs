use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// The engine a game data tree targets, which decides how its configs are read where the engines disagree.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayEngine {
  /// OpenXRay and the stock Call of Pripyat engine (`xray-16`).
  #[default]
  Vanilla,
  /// Anomaly's Monolith engine (`xray-monolith`) and the builds that share its readers.
  Extended,
}

impl XrayEngine {
  /// Every engine, in declaration order.
  pub const ALL: &'static [Self] = &[Self::Vanilla, Self::Extended];

  /// The name the engine is spelled with on a command line and in a setting.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Vanilla => "vanilla",
      Self::Extended => "extended",
    }
  }
}

impl Display for XrayEngine {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    formatter.write_str(self.as_str())
  }
}

impl FromStr for XrayEngine {
  type Err = String;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    Self::ALL
      .iter()
      .copied()
      .find(|engine| engine.as_str() == value)
      .ok_or_else(|| format!("Unknown engine '{value}', expected one of: vanilla, extended"))
  }
}

#[cfg(test)]
mod tests {
  use super::XrayEngine;

  #[test]
  fn spells_each_engine_as_it_parses_and_serializes() {
    for engine in XrayEngine::ALL {
      assert_eq!(engine.as_str().parse::<XrayEngine>(), Ok(*engine));
      assert_eq!(
        serde_json::to_string(engine).expect("an engine serializes"),
        format!("\"{engine}\"")
      );
    }
  }

  #[test]
  fn refuses_an_engine_it_does_not_name() {
    assert!("anomaly".parse::<XrayEngine>().is_err());
  }
}
