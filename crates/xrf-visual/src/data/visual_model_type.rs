/// Visual kinds an OGF header can declare, `enum MT` in `xrCore/FMesh.hpp`.
///
/// Kept internal: a consumer only ever displays the kind, so a description carries the raw byte and
/// the engine's own identifier for it. The packer is the only code that branches on the kind, to
/// decide whether a submesh needs its progressive detail table to be drawn correctly.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VisualModelType {
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

impl VisualModelType {
  pub(crate) const fn from_raw(raw: u8) -> Option<Self> {
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
  pub(crate) fn label(raw: u8) -> String {
    match Self::from_raw(raw) {
      Some(model_type) => String::from(model_type.name()),
      None => format!("MT_UNKNOWN({raw})"),
    }
  }

  /// Whether geometry of this kind is a sliding window over its index buffer.
  ///
  /// A progressive submesh stores every detail level in one buffer, so drawing all of it stacks the
  /// coarse shells over the fine one. Such a submesh is only drawable through its detail table.
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
