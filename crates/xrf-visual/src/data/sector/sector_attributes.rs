use xrf_level::LevelVertexLayout;

/// Which attributes a sector's vertices carry. The hemisphere term rides in the normal's fourth byte and the base
/// coordinate's low bytes in the tangent's and binormal's, so each comes with the direction carrying it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SectorAttributes {
  pub normals: bool,
  pub tangents: bool,
  pub binormals: bool,
  pub uvs: bool,
  pub lightmap_uvs: bool,
}

impl SectorAttributes {
  /// Every attribute a level's vertices can carry, for a caller that wants whatever a sector has.
  pub const fn all() -> Self {
    Self {
      binormals: true,
      lightmap_uvs: true,
      normals: true,
      tangents: true,
      uvs: true,
    }
  }

  /// What one declaration carries.
  pub const fn of(layout: &LevelVertexLayout) -> Self {
    Self {
      binormals: layout.get_binormal_offset().is_some(),
      lightmap_uvs: layout.get_lightmap_coordinate_offset().is_some(),
      normals: layout.get_normal_offset().is_some(),
      tangents: layout.get_tangent_offset().is_some(),
      uvs: layout.get_texture_coordinate_offset().is_some(),
    }
  }

  /// Widens what the sector carries by what one more declaration does.
  pub fn widen(&mut self, layout: &LevelVertexLayout) {
    let carried: Self = Self::of(layout);

    self.binormals |= carried.binormals;
    self.lightmap_uvs |= carried.lightmap_uvs;
    self.normals |= carried.normals;
    self.tangents |= carried.tangents;
    self.uvs |= carried.uvs;
  }

  /// What both this and `other` carry, which is what a pack is worth writing.
  #[must_use]
  pub const fn intersect(self, other: Self) -> Self {
    Self {
      binormals: self.binormals && other.binormals,
      lightmap_uvs: self.lightmap_uvs && other.lightmap_uvs,
      normals: self.normals && other.normals,
      tangents: self.tangents && other.tangents,
      uvs: self.uvs && other.uvs,
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::data::sector::sector_attributes::SectorAttributes;

  #[test]
  fn keeps_only_what_both_the_sector_and_the_caller_carry() {
    let carried: SectorAttributes = SectorAttributes {
      lightmap_uvs: true,
      normals: true,
      tangents: true,
      ..SectorAttributes::default()
    };
    let wanted: SectorAttributes = SectorAttributes {
      normals: true,
      tangents: true,
      uvs: true,
      ..SectorAttributes::default()
    };

    assert_eq!(
      carried.intersect(wanted),
      SectorAttributes {
        normals: true,
        tangents: true,
        ..SectorAttributes::default()
      },
      "a caller cannot ask for what the sector does not carry, nor a sector send what nothing draws"
    );
  }

  #[test]
  fn wanting_everything_leaves_what_the_sector_carries_untouched() {
    let carried: SectorAttributes = SectorAttributes {
      lightmap_uvs: true,
      normals: true,
      ..SectorAttributes::default()
    };

    assert_eq!(carried.intersect(SectorAttributes::all()), carried);
  }
}
