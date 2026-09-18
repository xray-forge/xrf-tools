use std::collections::BTreeMap;

use serde::Serialize;
use xrf_level::PsStaticPlacement;

/// One particle effect a level plants, and how widely.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelPsStaticEffect {
  /// The effect played, which names a definition inside `particles.xr` rather than a file, so it crosses as text.
  pub name: String,
  pub placements: usize,
  /// Placements of this effect that only some multiplayer modes load, which a single-player session never plays.
  pub restricted: usize,
}

impl ArchiveLevelPsStaticEffect {
  /// Every effect a level plants, by name, in the order a reader would look them up.
  pub fn of_all(placements: &[PsStaticPlacement]) -> Vec<Self> {
    let mut counted: BTreeMap<&str, (usize, usize)> = BTreeMap::new();

    for placement in placements {
      let entry: &mut (usize, usize) = counted.entry(placement.effect.as_str()).or_default();

      entry.0 += 1;

      if !placement.is_every_game_type() {
        entry.1 += 1;
      }
    }

    counted
      .into_iter()
      .map(|(name, (placements, restricted))| Self {
        name: name.to_owned(),
        placements,
        restricted,
      })
      .collect()
  }

  /// Whether every placement of this effect is one a single-player session loads.
  pub const fn is_every_game_type(&self) -> bool {
    self.restricted == 0
  }
}
