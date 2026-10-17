/// What a texture is sampled as, which decides what stands in for it while it loads or where it cannot be had.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextureRole {
  /// A diffuse, stood in for by a checker where it cannot be had, so the gap is seen.
  Base,
  /// A detail, neutral at mid grey since it is applied doubled.
  Detail,
  /// A bump's normal and gloss, neutral along the surface normal.
  Bump,
  /// A bump's error companion, neutral at mid grey.
  BumpCompanion,
  /// A lightmap, neutral at white: the open sky and full sun.
  Hemi,
  /// An impostor's `_nm` companion, neutral facing the eye under the open sky.
  ImpostorCompanion,
  /// A spot's projector, neutral at white: the light passing whole.
  Projector,
  /// A terrain's mask, neutral at white: its four details weighed alike.
  TerrainMask,
  /// A terrain detail's height, neutral at a half, the plane: the installation's own (`<detail>_height`, red), or
  /// derived from the detail's bump where it brings none.
  TerrainHeight,
  /// A water's normal map, neutral facing straight out of the surface: two reads of it summed less one give its normal.
  WaterNormal,
}

impl TextureRole {
  /// Every role, in declaration order, so `role as usize` indexes what is kept a role.
  pub const ALL: [TextureRole; 10] = [
    TextureRole::Base,
    TextureRole::Detail,
    TextureRole::Bump,
    TextureRole::BumpCompanion,
    TextureRole::Hemi,
    TextureRole::ImpostorCompanion,
    TextureRole::Projector,
    TextureRole::TerrainMask,
    TextureRole::TerrainHeight,
    TextureRole::WaterNormal,
  ];

  /// What a terrain detail's height is named by beside the detail, as the third-party pack names it, and its bump.
  pub const HEIGHT_SUFFIX: &'static str = "_height";
  pub const BUMP_SUFFIX: &'static str = "_bump";

  /// The colour it is drawn as while its texture loads.
  pub const fn get_neutral(self) -> [u8; 4] {
    match self {
      TextureRole::Base | TextureRole::Hemi | TextureRole::Projector | TextureRole::TerrainMask => [255, 255, 255, 255],
      TextureRole::Detail | TextureRole::TerrainHeight => [128, 128, 128, 128],
      TextureRole::Bump => [23, 255, 128, 128],
      TextureRole::BumpCompanion => [128, 128, 128, 0],
      TextureRole::ImpostorCompanion => [128, 128, 0, 255],
      TextureRole::WaterNormal => [128, 128, 255, 255],
    }
  }
}
