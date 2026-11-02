use std::collections::BTreeMap;

use serde::Serialize;
use xrf_level::LevelGameRPoint;

/// One kind of respawn point a level declares, and how many of them there are.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelGameSpawn {
  /// What the points spawn, named where the engine names the kind.
  pub label: Option<String>,
  /// The stored kind, kept because a file may carry one the engine gives no name.
  pub kind: u8,
  pub points: usize,
  /// Points of this kind naming a spawn preset, which only item points do.
  pub profiled: usize,
}

impl ArchiveLevelGameSpawn {
  /// Every kind of point a level declares, in the order the engine numbers them.
  pub fn of_all(rpoints: &[LevelGameRPoint]) -> Vec<Self> {
    let mut counted: BTreeMap<u8, (usize, usize)> = BTreeMap::new();

    for rpoint in rpoints {
      let entry: &mut (usize, usize) = counted.entry(rpoint.kind).or_default();

      entry.0 += 1;

      if !rpoint.profile.is_empty() {
        entry.1 += 1;
      }
    }

    counted
      .into_iter()
      .map(|(kind, (points, profiled))| Self {
        label: rpoints
          .iter()
          .find(|rpoint| rpoint.kind == kind)
          .and_then(LevelGameRPoint::get_kind_label)
          .map(ToOwned::to_owned),
        kind,
        points,
        profiled,
      })
      .collect()
  }
}
