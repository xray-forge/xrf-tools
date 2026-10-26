use crate::texture::thm_texture_flag::ThmTextureFlag;

/// How a live detail association is applied, from the texture param flags (`TextureDescrManager.cpp:175`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThmDetailUsage {
  Diffuse,
  Bump,
  DiffuseAndBump,
}

impl ThmDetailUsage {
  /// The flags that apply a detail association this way, the ones
  /// [`ThmFile::used_detail_usage`](crate::ThmFile::used_detail_usage) reads it from.
  pub const fn get_flags(self) -> &'static [ThmTextureFlag] {
    match self {
      Self::Diffuse => &[ThmTextureFlag::DiffuseDetail],
      Self::Bump => &[ThmTextureFlag::BumpDetail],
      Self::DiffuseAndBump => &[ThmTextureFlag::DiffuseDetail, ThmTextureFlag::BumpDetail],
    }
  }
}
