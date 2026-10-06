use std::fmt::{Display, Formatter, Result as FmtResult};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::xray_engine::XrayEngine;
use crate::xray_engine_resolution::XrayEngineResolution;

/// Which engine a caller asks a tree to be read as: one it names, or whatever the tree shows.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayEngineChoice {
  /// Detected from the tree's installation layout and data.
  #[default]
  Auto,
  /// OpenXRay and the stock Call of Pripyat engine, whatever the tree shows.
  Vanilla,
  /// Anomaly's Monolith engine, whatever the tree shows.
  Extended,
}

impl XrayEngineChoice {
  /// Every choice, in declaration order.
  pub const ALL: &'static [Self] = &[Self::Auto, Self::Vanilla, Self::Extended];

  /// The name the choice is spelled with on a command line and in a request.
  pub const fn as_str(self) -> &'static str {
    match self {
      Self::Auto => "auto",
      Self::Vanilla => "vanilla",
      Self::Extended => "extended",
    }
  }

  /// The engine this choice names, or `None` for one left to detection.
  pub const fn get_named(self) -> Option<XrayEngine> {
    match self {
      Self::Auto => None,
      Self::Vanilla => Some(XrayEngine::Vanilla),
      Self::Extended => Some(XrayEngine::Extended),
    }
  }

  /// The engine this choice comes to, running `detect` only when nothing is named.
  pub fn resolve(self, detect: impl FnOnce() -> XrayEngineResolution) -> XrayEngineResolution {
    match self.get_named() {
      Some(engine) => XrayEngineResolution::named(engine),
      None => detect(),
    }
  }
}

impl Display for XrayEngineChoice {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
    formatter.write_str(self.as_str())
  }
}

impl FromStr for XrayEngineChoice {
  type Err = String;

  fn from_str(value: &str) -> Result<Self, Self::Err> {
    Self::ALL
      .iter()
      .copied()
      .find(|choice| choice.as_str() == value)
      .ok_or_else(|| format!("Unknown engine '{value}', expected one of: auto, vanilla, extended"))
  }
}

#[cfg(test)]
mod tests {
  use super::XrayEngineChoice;
  use crate::{XrayEngine, XrayEngineEvidence, XrayEngineResolution};

  #[test]
  fn spells_each_choice_as_it_parses_and_serializes() {
    for choice in XrayEngineChoice::ALL {
      assert_eq!(choice.as_str().parse::<XrayEngineChoice>(), Ok(*choice));
      assert_eq!(
        serde_json::to_string(choice).expect("a choice serializes"),
        format!("\"{choice}\"")
      );
    }
  }

  #[test]
  fn defaults_to_detection() {
    assert_eq!(XrayEngineChoice::default(), XrayEngineChoice::Auto);
    assert!("anomaly".parse::<XrayEngineChoice>().is_err());
  }

  #[test]
  fn resolves_a_named_engine_without_detecting() {
    let resolution: XrayEngineResolution =
      XrayEngineChoice::Extended.resolve(|| panic!("a named engine is never detected"));

    assert_eq!(resolution.engine, XrayEngine::Extended);
    assert_eq!(resolution.evidence, XrayEngineEvidence::Named);
    assert_eq!(resolution.subject, None);
  }

  #[test]
  fn resolves_auto_by_detecting() {
    let detected: XrayEngineResolution =
      XrayEngineResolution::detected(XrayEngine::Extended, XrayEngineEvidence::AtmosfearCycles, "graphs");

    assert_eq!(XrayEngineChoice::Auto.resolve(|| detected.clone()), detected);
  }
}
