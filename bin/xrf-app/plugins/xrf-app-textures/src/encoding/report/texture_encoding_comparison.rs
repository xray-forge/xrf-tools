use serde::Serialize;
use xrf_app_core::session::SessionId;
use xrf_job::JobOutcome;
use xrf_vfs::XrayRoots;

use crate::describe::TextureSource;
use crate::encoding::{TextureEncodingCurrent, TextureEncodingReport};

/// Every candidate weighed against one texture, with the texture itself for a baseline.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEncodingComparison {
  /// The comparison holding the encodes, which a save names to claim one.
  pub session_id: SessionId,
  pub source: TextureSource,
  pub roots: XrayRoots,
  /// Whether every candidate was weighed or the run stopped because it was asked to.
  pub outcome: JobOutcome,
  /// What to call the texture the encodes were made from.
  pub reference: String,
  pub current: TextureEncodingCurrent,
  /// The candidates weighed, in the order `DdsEncodeCandidate::ALL` lists them, cheapest first.
  pub candidates: Vec<TextureEncodingReport>,
}
