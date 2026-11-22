use log::Level;
use serde::{Deserialize, Serialize};

/// A level the webview logs at, its most verbose being debug.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum UiLogLevel {
  Error,
  Warn,
  Info,
  Debug,
}

impl From<UiLogLevel> for Level {
  fn from(level: UiLogLevel) -> Self {
    match level {
      UiLogLevel::Error => Self::Error,
      UiLogLevel::Warn => Self::Warn,
      UiLogLevel::Info => Self::Info,
      UiLogLevel::Debug => Self::Debug,
    }
  }
}

/// Trace, which the webview does not log at, reads as debug: a filter letting trace through lets debug through.
impl From<Level> for UiLogLevel {
  fn from(level: Level) -> Self {
    match level {
      Level::Error => Self::Error,
      Level::Warn => Self::Warn,
      Level::Info => Self::Info,
      Level::Debug | Level::Trace => Self::Debug,
    }
  }
}
