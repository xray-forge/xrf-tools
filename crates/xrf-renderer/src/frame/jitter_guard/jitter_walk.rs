use std::collections::{HashMap, HashSet};

use xrf_renderer_core::{GraphOwner, GraphPassKind, GraphPassReport, GraphReport};

use crate::frame::jitter_guard::JitterGuard;

/// The jitter as the walk over a frame's passes has followed it so far: each texture carrying a view's jitter, by the
/// view, and the views that have resolved.
pub(super) struct JitterWalk {
  jittered: HashMap<usize, GraphOwner>,
  resolved: HashSet<GraphOwner>,
}

impl JitterWalk {
  /// The walk before the first pass: every texture tagged `JitterGuard::JITTERED` carries the jitter of the view it
  /// was declared under, and no view has resolved.
  pub(super) fn of(report: &GraphReport) -> Self {
    Self {
      jittered: report
        .textures
        .iter()
        .enumerate()
        .filter(|(_, texture)| texture.tags.contains(&JitterGuard::JITTERED))
        .map(|(index, texture)| (index, texture.owner))
        .collect(),
      resolved: HashSet::new(),
    }
  }

  /// Follows the jitter through `pass`, answering the jittered textures it reads after their view's resolve, as places
  /// in the report's `textures`.
  ///
  /// Its reads are judged before its writes change the state. Before a view's resolve, reading a texture carrying the
  /// view's jitter carries it on to everything the pass writes, as does a raster pass drawing into one, whose
  /// attachments are one rasterisation. The resolve writes nothing jittered, whatever it read. After the resolve, a
  /// read carries nothing on, and a pass drawing a jittered texture anew without reading it leaves it unjittered.
  pub(super) fn step(&mut self, pass: &GraphPassReport) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    let mut carried: Option<GraphOwner> = None;

    for texture in &pass.reads {
      match self.jittered.get(texture) {
        Some(view) if self.resolved.contains(view) => found.push(*texture),
        Some(view) => carried = Some(*view),
        None => {}
      }
    }

    if pass.kind == GraphPassKind::Raster {
      carried = carried.or_else(|| {
        pass
          .writes
          .iter()
          .filter_map(|texture| self.jittered.get(texture))
          .find(|view| !self.resolved.contains(*view))
          .copied()
      });
    }

    if pass.tags.contains(&JitterGuard::RESOLVES) {
      self.resolved.insert(pass.owner);

      for texture in &pass.writes {
        self.jittered.remove(texture);
      }
    } else if let Some(view) = carried {
      self.jittered.extend(pass.writes.iter().map(|texture| (*texture, view)));
    } else {
      for texture in pass.writes.iter().filter(|texture| !pass.reads.contains(texture)) {
        if self
          .jittered
          .get(texture)
          .is_some_and(|view| self.resolved.contains(view))
        {
          self.jittered.remove(texture);
        }
      }
    }

    found
  }
}
