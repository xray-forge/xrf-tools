/// The reflection denoiser's stages, each a fragment entry point of `frame/reflections.wgsl`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReflectionStage {
  Reproject,
  Average,
  Prefilter,
  Resolve,
  Upscale,
  Guide,
}

impl ReflectionStage {
  /// The stage's entry point.
  pub const fn get_entry(self) -> &'static str {
    match self {
      Self::Reproject => "fs_reproject",
      Self::Average => "fs_average",
      Self::Prefilter => "fs_prefilter",
      Self::Resolve => "fs_resolve",
      Self::Upscale => "fs_upscale",
      Self::Guide => "fs_guide",
    }
  }
}
