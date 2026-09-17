use serde::Serialize;
use xrf_db::{GAMEMTL_PAIR_PROPERTIES, GameMtlPair};

/// One thing a material pairing can declare, and how many pairings declare it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveGameMtlProperty {
  pub name: String,
  pub pairs: usize,
}

impl ArchiveGameMtlProperty {
  /// What the pairings of a library declare, counted per property.
  ///
  /// Counted rather than listed: a library carries up to 1,550 pairings and no reader goes through them one by one.
  pub fn of_all(pairs: &[GameMtlPair]) -> Vec<Self> {
    GAMEMTL_PAIR_PROPERTIES
      .into_iter()
      .map(|(mask, name)| Self {
        name: name.to_owned(),
        pairs: pairs.iter().filter(|pair| pair.own_properties & mask != 0).count(),
      })
      .collect()
  }
}
