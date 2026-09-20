use xrf_level::LevelVertexLayout;

/// Which attributes a sector's vertices carry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SectorAttributes {
  pub normals: bool,
  pub tangents: bool,
  pub binormals: bool,
  pub uvs: bool,
  pub lightmap_uvs: bool,
  pub colors: bool,
  /// Hemisphere occlusion, which rides in the normal's fourth byte and is therefore available exactly with it.
  pub hemi: bool,
}

impl SectorAttributes {
  /// Every attribute a level's vertices can carry, for a caller that wants whatever a sector has.
  pub const fn all() -> Self {
    Self {
      binormals: true,
      colors: true,
      hemi: true,
      lightmap_uvs: true,
      normals: true,
      tangents: true,
      uvs: true,
    }
  }

  /// Widens what the sector carries by what one more declaration does.
  pub fn widen(&mut self, layout: &LevelVertexLayout) {
    self.binormals |= layout.get_binormal_offset().is_some();
    self.colors |= layout.get_color_offset().is_some();
    self.lightmap_uvs |= layout.get_lightmap_coordinate_offset().is_some();
    self.normals |= layout.get_normal_offset().is_some();
    self.tangents |= layout.get_tangent_offset().is_some();
    self.uvs |= layout.get_texture_coordinate_offset().is_some();
    self.hemi |= layout.get_normal_offset().is_some();
  }

  /// What both this and `other` carry, which is what a pack is worth writing.
  #[must_use]
  pub const fn intersect(self, other: Self) -> Self {
    Self {
      binormals: self.binormals && other.binormals,
      colors: self.colors && other.colors,
      hemi: self.hemi && other.hemi,
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
      colors: true,
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
