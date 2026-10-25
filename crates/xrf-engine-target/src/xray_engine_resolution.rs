use serde::{Deserialize, Serialize};

use crate::xray_engine::XrayEngine;
use crate::xray_engine_evidence::XrayEngineEvidence;

/// The engine a tree is read as, and what decided it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayEngineResolution {
  pub engine: XrayEngine,
  pub evidence: XrayEngineEvidence,
  /// The file or logical path that showed it, where one did.
  pub subject: Option<String>,
}

impl XrayEngineResolution {
  /// An engine the caller named.
  pub fn named(engine: XrayEngine) -> Self {
    Self {
      engine,
      evidence: XrayEngineEvidence::Named,
      subject: None,
    }
  }

  /// An engine a file of the tree showed.
  pub fn detected(engine: XrayEngine, evidence: XrayEngineEvidence, subject: impl Into<String>) -> Self {
    Self {
      engine,
      evidence,
      subject: Some(subject.into()),
    }
  }

  /// Vanilla, for a tree showing nothing else.
  pub fn undetected() -> Self {
    Self {
      engine: XrayEngine::Vanilla,
      evidence: XrayEngineEvidence::NoSigns,
      subject: None,
    }
  }

  /// The engine and why, as one line: `extended (Anomaly executables found)`.
  pub fn describe(&self) -> String {
    format!("{} ({})", self.engine, self.evidence.describe())
  }
}

#[cfg(test)]
mod tests {
  use super::XrayEngineResolution;
  use crate::{XrayEngine, XrayEngineEvidence};

  #[test]
  fn describes_the_engine_and_what_decided_it() {
    assert_eq!(
      XrayEngineResolution::detected(XrayEngine::Extended, XrayEngineEvidence::AnomalyExecutables, "launcher")
        .describe(),
      "extended (Anomaly executables found)"
    );
    assert_eq!(
      XrayEngineResolution::named(XrayEngine::Vanilla).describe(),
      "vanilla (named)"
    );
    assert_eq!(
      XrayEngineResolution::undetected().describe(),
      "vanilla (no Anomaly or Atmosfear signs)"
    );
  }
}
