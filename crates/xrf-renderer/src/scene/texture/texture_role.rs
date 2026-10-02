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
}

impl TextureRole {
  /// Every role, in declaration order, so `role as usize` indexes what is kept a role.
  pub const ALL: [TextureRole; 6] = [
    TextureRole::Base,
    TextureRole::Detail,
    TextureRole::Bump,
    TextureRole::BumpCompanion,
    TextureRole::Hemi,
    TextureRole::ImpostorCompanion,
  ];

  /// The colour it is drawn as while its texture loads.
  pub const fn get_neutral(self) -> [u8; 4] {
    match self {
      TextureRole::Base | TextureRole::Hemi => [255, 255, 255, 255],
      TextureRole::Detail => [128, 128, 128, 128],
      TextureRole::Bump => [23, 255, 128, 128],
      TextureRole::BumpCompanion => [128, 128, 128, 0],
      TextureRole::ImpostorCompanion => [128, 128, 0, 255],
    }
  }
}
