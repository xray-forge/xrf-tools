/// The ambient occlusion's stages, in the order a frame draws them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AmbientOcclusionStage {
  /// The search, into the first target.
  Search,
  /// The denoise across, into the second.
  DenoiseAcross,
  /// The denoise down, back into the first, which combine reads.
  DenoiseBack,
}

impl AmbientOcclusionStage {
  pub const ALL: [Self; 3] = [Self::Search, Self::DenoiseAcross, Self::DenoiseBack];

  /// The stage's pass, as the frame graph names it.
  pub const fn get_name(self) -> &'static str {
    match self {
      Self::Search => "ambient occlusion",
      Self::DenoiseAcross => "occlusion denoise",
      Self::DenoiseBack => "occlusion denoise back",
    }
  }

  /// The occlusion target the stage reads (the other than it draws into) and the one it draws into.
  pub const fn get_targets(self) -> (usize, usize) {
    match self {
      Self::Search | Self::DenoiseBack => (0, 0),
      Self::DenoiseAcross => (1, 1),
    }
  }
}
