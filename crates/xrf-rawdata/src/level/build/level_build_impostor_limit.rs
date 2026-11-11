use crate::level::build::level_build_atlas::{LevelBuildAtlas, LevelBuildAtlasSizes};

/// How large a build lets the impostor atlas, `level_lods.dds` and `_nm`, grow.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LevelBuildImpostorLimit {
  /// SDK 0.7's, 2048 a side, 128 strips of 512 by 64; a level of more is refused, as that Level Editor refuses it.
  Sdk,
  /// 4096 a side, 512 strips, as the editors Anomaly's and GAMMA's largest levels were built with grew it: their
  /// `jupiter` and `pripyat` ship a 4096 by 2048 atlas of 130 to 153 strips.
  #[default]
  Extended,
}

impl LevelBuildImpostorLimit {
  /// The sizes the atlas may take under this limit.
  pub(crate) fn get_sizes(self) -> LevelBuildAtlasSizes {
    match self {
      Self::Sdk => LevelBuildAtlas::LOD_SIZES,
      Self::Extended => LevelBuildAtlas::EXTENDED_LOD_SIZES,
    }
  }
}
