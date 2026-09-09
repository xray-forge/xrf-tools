use serde::{Deserialize, Serialize};
use xrf_vfs::XrayRoots;

use crate::plugins::configs::session_id::ConfigsSessionId;

/// What opening a configs project for browsing was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsOpenRequest {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
  /// Whether to resolve with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
}

/// Which config of which open a reader wants.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsReadDocumentRequest {
  /// The open this read is addressed to; a read naming a replaced one is refused rather than answered.
  pub session_id: ConfigsSessionId,
  /// Engine identity of the config to read.
  pub path: String,
}

/// What a config verification was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsVerifyRequest {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
  /// Whether to resolve with the Monolith/Anomaly DLTX patch dialect.
  pub is_dltx: bool,
}

/// What a config formatting run, or a check of one, was asked to do.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsFormatRequest {
  /// Trees to search, and how each is read.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
}

/// Which resolved root a reader wants, of which open.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsResolvedRequest {
  /// The open this read is addressed to; a read naming a replaced one is refused rather than answered.
  pub session_id: ConfigsSessionId,
  /// Engine identity of the entry point to resolve.
  pub entry: String,
}

/// Which sections of a resolved root a page wants.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct ConfigsReadSectionsRequest {
  /// The open this read is addressed to; a read naming a replaced one is refused rather than answered.
  pub session_id: ConfigsSessionId,
  /// Engine identity of the entry point the sections belong to.
  pub entry: String,
  /// Sections to read, as the index named them.
  pub names: Vec<String>,
}
