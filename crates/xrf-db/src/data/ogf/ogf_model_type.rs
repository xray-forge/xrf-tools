/// Visual kinds an OGF header can declare, `enum MT` in `xray-16/src/xrCore/FMesh.hpp:9`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OgfModelType {
  Normal,
  Hierarchy,
  Progressive,
  SkeletonAnim,
  SkeletonGeomdefPm,
  SkeletonGeomdefSt,
  Lod,
  TreeSt,
  ParticleEffect,
  ParticleGroup,
  SkeletonRigid,
  TreePm,
  FluidVolume,
}

impl OgfModelType {
  /// The kind a raw header byte names, or `None` for a byte the engine has no kind for.
  pub const fn from_raw(raw: u8) -> Option<Self> {
    match raw {
      0 => Some(Self::Normal),
      1 => Some(Self::Hierarchy),
      2 => Some(Self::Progressive),
      3 => Some(Self::SkeletonAnim),
      4 => Some(Self::SkeletonGeomdefPm),
      5 => Some(Self::SkeletonGeomdefSt),
      6 => Some(Self::Lod),
      7 => Some(Self::TreeSt),
      8 => Some(Self::ParticleEffect),
      9 => Some(Self::ParticleGroup),
      10 => Some(Self::SkeletonRigid),
      11 => Some(Self::TreePm),
      12 => Some(Self::FluidVolume),
      _ => None,
    }
  }

  /// Engine identifier for a raw model type byte, or a labelled unknown for a byte the engine has no
  /// name for. Unknown bytes keep their value visible rather than collapsing to one label.
  pub fn label(raw: u8) -> String {
    match Self::from_raw(raw) {
      Some(model_type) => String::from(model_type.name()),
      None => format!("MT_UNKNOWN({raw})"),
    }
  }

  /// Whether geometry of this kind is a sliding window over its index buffer.
  pub const fn is_progressive(self) -> bool {
    matches!(self, Self::Progressive | Self::SkeletonGeomdefPm | Self::TreePm)
  }

  const fn name(self) -> &'static str {
    match self {
      Self::Normal => "MT_NORMAL",
      Self::Hierarchy => "MT_HIERRARHY",
      Self::Progressive => "MT_PROGRESSIVE",
      Self::SkeletonAnim => "MT_SKELETON_ANIM",
      Self::SkeletonGeomdefPm => "MT_SKELETON_GEOMDEF_PM",
      Self::SkeletonGeomdefSt => "MT_SKELETON_GEOMDEF_ST",
      Self::Lod => "MT_LOD",
      Self::TreeSt => "MT_TREE_ST",
      Self::ParticleEffect => "MT_PARTICLE_EFFECT",
      Self::ParticleGroup => "MT_PARTICLE_GROUP",
      Self::SkeletonRigid => "MT_SKELETON_RIGID",
      Self::TreePm => "MT_TREE_PM",
      Self::FluidVolume => "MT_3DFLUIDVOLUME",
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::data::ogf::ogf_model_type::OgfModelType;

  #[test]
  fn test_label_names_every_engine_kind() {
    assert_eq!(OgfModelType::label(0), "MT_NORMAL");
    assert_eq!(OgfModelType::label(6), "MT_LOD");
    assert_eq!(OgfModelType::label(11), "MT_TREE_PM");
    assert_eq!(OgfModelType::label(12), "MT_3DFLUIDVOLUME");
  }

  #[test]
  fn test_label_keeps_an_unknown_byte_visible() {
    assert_eq!(OgfModelType::from_raw(13), None);
    assert_eq!(OgfModelType::label(13), "MT_UNKNOWN(13)");
  }

  #[test]
  fn test_progressive_kinds_are_sliding_windows() {
    assert!(OgfModelType::Progressive.is_progressive());
    assert!(OgfModelType::SkeletonGeomdefPm.is_progressive());
    assert!(OgfModelType::TreePm.is_progressive());
    assert!(!OgfModelType::TreeSt.is_progressive());
    assert!(!OgfModelType::Normal.is_progressive());
  }
}
