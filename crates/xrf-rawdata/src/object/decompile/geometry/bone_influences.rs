use xrf_ogf::OgfVertexLink;

/// The bones one point follows and how strongly, as an exported vertex's links give them back.
///
/// `CExportSkeleton` writes every vertex of a child in the child's widest layout, padding the links a vertex lacks:
/// two-link vertices repeat the first bone with the whole weight on the copy, and three- or four-link vertices repeat
/// it with no weight, the last link taking what the stored weights leave, which rounding can make a sliver above or
/// below nought. Summing each bone's weights and dropping the empty ones undoes the padding, so a point reads alike in
/// every child it was exported to. The weights are the ones `st_VertexWB::prepare_weights` normalised, at most the
/// four strongest; the scale they were authored at is not stored.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoneInfluences {
  /// Each bone and its weight, by ascending bone, then unused slots.
  influences: [(u16, f32); Self::CAPACITY],
  count: u8,
}

impl BoneInfluences {
  /// Bones a vertex can follow, as many as the widest skin layout stores.
  pub(crate) const CAPACITY: usize = 4;

  /// A summed weight at or below which a bone is padding the export wrote, not an influence: the remainder of weights
  /// that already sum to one differs from nought by rounding alone.
  const NEGLIGIBLE_WEIGHT: f32 = 1e-6;

  /// The influences `links` stand for, padding undone.
  pub(crate) fn of_links(links: &[OgfVertexLink]) -> Self {
    let mut summed: [(u16, f32); Self::CAPACITY] = [(0, 0.0); Self::CAPACITY];
    let mut count: usize = 0;

    for link in links.iter().take(Self::CAPACITY) {
      match summed[..count].iter_mut().find(|(bone, _)| *bone == link.bone) {
        Some((_, weight)) => *weight += link.weight,
        None => {
          summed[count] = (link.bone, link.weight);
          count += 1;
        }
      }
    }

    let summed: &[(u16, f32)] = &summed[..count];
    let mut influences: Self = Self {
      influences: [(0, 0.0); Self::CAPACITY],
      count: 0,
    };

    for influence in summed.iter().filter(|(_, weight)| *weight > Self::NEGLIGIBLE_WEIGHT) {
      influences.influences[influences.len()] = *influence;
      influences.count += 1;
    }

    // Rounding cannot leave every weight negligible, but a malformed vertex could: keep its strongest bone.
    if influences.count == 0
      && let Some(strongest) = summed.iter().max_by(|first, second| first.1.total_cmp(&second.1))
    {
      influences.influences[0] = *strongest;
      influences.count = 1;
    }

    let count: usize = influences.len();

    influences.influences[..count].sort_by_key(|(bone, _)| *bone);
    influences
  }

  /// A point following `bone` alone.
  #[cfg(test)]
  pub(crate) fn of_bone(bone: u16) -> Self {
    Self::of_links(&[OgfVertexLink { bone, weight: 1.0 }])
  }

  /// Each bone and its weight, by ascending bone.
  pub(crate) fn iter(&self) -> impl Iterator<Item = (u16, f32)> + '_ {
    self.influences[..self.len()].iter().copied()
  }

  /// The bones, padded with `u32::MAX`, which with a position decides a welded point.
  pub(crate) fn get_bone_key(&self) -> [u32; Self::CAPACITY] {
    std::array::from_fn(|index| {
      self.influences[..self.len()]
        .get(index)
        .map_or(u32::MAX, |(bone, _)| u32::from(*bone))
    })
  }

  fn len(&self) -> usize {
    usize::from(self.count)
  }
}
