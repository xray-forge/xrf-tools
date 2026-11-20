use xrf_ogf::OgfVertexLink;

/// The bones one point follows and how strongly, as an exported vertex's links give them back.
///
/// `CExportSkeleton` pads every vertex to its child's widest layout by repeating the first bone, its weight whole,
/// none, or a rounding sliver; summing each bone's weights and dropping empty ones undoes that, so a point reads alike
/// in every child. The weights are `st_VertexWB::prepare_weights`'s normalised strongest four, not the authored ones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoneInfluences {
  /// Each bone and its weight, by ascending bone, then unused slots.
  influences: [(u16, f32); Self::CAPACITY],
  count: u8,
}

impl BoneInfluences {
  /// Bones a vertex can follow, as many as the widest skin layout stores.
  pub(crate) const CAPACITY: usize = 4;

  /// A static vertex's: it follows no bone.
  pub(crate) const NONE: Self = Self {
    influences: [(0, 0.0); Self::CAPACITY],
    count: 0,
  };

  /// A summed weight at or below which a bone is padding rather than an influence.
  const NEGLIGIBLE_WEIGHT: f32 = 1e-6;

  /// How far one point's weights may differ between children of different layouts, where the export derives the last
  /// link's weight from the others: rounding, far below any two points' authored weights.
  const WEIGHT_TOLERANCE: f32 = 1e-4;

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

  /// Whether `other` follows the same bones as strongly, but for the rounding two link layouts give one point.
  pub(crate) fn is_alike(&self, other: &Self) -> bool {
    self.count == other.count
      && self
        .iter()
        .zip(other.iter())
        .all(|((bone, weight), (other_bone, other_weight))| {
          bone == other_bone && (weight - other_weight).abs() <= Self::WEIGHT_TOLERANCE
        })
  }

  fn len(&self) -> usize {
    usize::from(self.count)
  }
}
