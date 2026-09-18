use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::level::level_visual::LevelVisual;
use crate::level::level_visuals_chunk::LevelVisualsChunk;

/// What one sector reaches from its root, and what stopped the walk.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSectorComposition {
  /// Visuals reached from the root that draw geometry, by their index in the run, without repeats.
  pub drawables: Vec<u32>,
  /// Visuals reached that draw nothing themselves, the root among them.
  pub hierarchies: Vec<u32>,
  /// Ids reached that no visual answers to, which the engine would hand to `getVisual` regardless.
  pub unknown: Vec<u32>,
  /// Ids reached a second time, which is a link graph that is not a tree.
  pub revisited: Vec<u32>,
}

impl LevelSectorComposition {
  /// Walks one sector's root and records everything it reaches.
  pub fn of(visuals: &LevelVisualsChunk, root: u32) -> Self {
    let mut composition: Self = Self::default();
    let mut seen: BTreeSet<u32> = BTreeSet::new();
    let mut pending: Vec<u32> = vec![root];

    while let Some(index) = pending.pop() {
      if !seen.insert(index) {
        composition.revisited.push(index);

        continue;
      }

      let Some(visual) = visuals.visuals.get(index as usize) else {
        composition.unknown.push(index);

        continue;
      };

      if visual.is_drawable() {
        composition.drawables.push(index);
      } else {
        composition.hierarchies.push(index);
      }

      pending.extend(Self::children_of(visual));
    }

    composition.drawables.sort_unstable();
    composition.hierarchies.sort_unstable();
    composition.revisited.sort_unstable();
    composition.unknown.sort_unstable();

    composition
  }

  /// How many visuals the walk reached in total, however they were classified.
  pub fn count_reached(&self) -> usize {
    self.drawables.len() + self.hierarchies.len() + self.unknown.len()
  }

  /// Whether everything the walk reached is a visual the level holds.
  pub const fn is_resolved(&self) -> bool {
    self.unknown.is_empty()
  }

  /// The children of a visual, which a drawable one may carry as well.
  fn children_of(visual: &LevelVisual) -> &[u32] {
    &visual.children
  }
}
