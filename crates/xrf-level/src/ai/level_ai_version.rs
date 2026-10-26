use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};

use crate::ai::level_ai_node_layout::LevelAiNodeLayout;

/// The node layouts a `level.ai` has been written in, `xrAI_Versions` (`xray-16/src/Common/LevelStructure.hpp`).
///
/// OpenXRay loads every one of them, converting the older ones on load (`CLevelGraphManager`); every shipped file of
/// the workspace trees is [`Self::CsCop`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelAiVersion {
  /// Shadow of Chernobyl: 23-bit links and one cover set, `NodeCompressed7`.
  Soc = 8,
  /// Early Clear Sky builds on the Shadow of Chernobyl engine, laid out as [`Self::Soc`].
  Priquel = 9,
  /// Clear Sky and Call of Pripyat: high and low cover, `NodeCompressed10`.
  CsCop = 10,
  /// 25-bit links, `NodeCompressed11`.
  Borsht = 11,
  /// 25-bit links and a whole `u32` for the cell, `NodeCompressed12`.
  BorshtBig = 12,
  /// 26-bit links and no light bits, `NodeCompressed13`; OpenXRay's current version.
  Skyloader = 13,
}

impl LevelAiVersion {
  /// The version a header names.
  ///
  /// # Errors
  ///
  /// Returns an error for a version the engine refuses to load, `ASSERT_XRAI_VERSION_MATCH`.
  pub fn from_number(version: u32) -> XrfResult<Self> {
    match version {
      8 => Ok(Self::Soc),
      9 => Ok(Self::Priquel),
      10 => Ok(Self::CsCop),
      11 => Ok(Self::Borsht),
      12 => Ok(Self::BorshtBig),
      13 => Ok(Self::Skyloader),
      _ => Err(XrfError::new_not_implemented_error(format!(
        "Unexpected level AI-map version {version}, the engine loads versions 8 to 13"
      ))),
    }
  }

  /// Bytes one node of this version occupies.
  pub const fn get_node_size(self) -> u64 {
    self.get_node_layout().size as u64
  }

  /// Where a node of this version keeps each field.
  pub(crate) const fn get_node_layout(self) -> LevelAiNodeLayout {
    match self {
      Self::Soc | Self::Priquel => LevelAiNodeLayout::NODE_7,
      Self::CsCop => LevelAiNodeLayout::NODE_10,
      Self::Borsht => LevelAiNodeLayout::NODE_11,
      Self::BorshtBig => LevelAiNodeLayout::NODE_12,
      Self::Skyloader => LevelAiNodeLayout::NODE_13,
    }
  }
}
