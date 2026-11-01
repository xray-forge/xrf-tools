/// The upscale pass's stages, by the pipeline each draws with.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UpscaleStage {
  /// EASU's upscale, of a frame smaller than the viewport.
  Easu,
  /// RCAS's sharpening of an upscaled frame.
  Rcas,
  /// The area-averaged downscale of a frame larger than the viewport.
  Downsample,
}

impl UpscaleStage {
  pub const ALL: [Self; 3] = [Self::Easu, Self::Rcas, Self::Downsample];
  pub const COUNT: usize = Self::ALL.len();

  /// Its place among the stages, which its pipeline is kept at.
  pub const fn get_index(self) -> usize {
    self as usize
  }

  /// The stage's pass, as the frame graph names it.
  pub const fn get_name(self) -> &'static str {
    match self {
      Self::Easu => "upscale",
      Self::Rcas => "sharpen",
      Self::Downsample => "downsample",
    }
  }

  /// The stage's fragment entry point of `frame/upscale.wgsl`.
  pub const fn get_entry(self) -> &'static str {
    match self {
      Self::Easu => "fs_easu",
      Self::Rcas => "fs_rcas",
      Self::Downsample => "fs_downsample",
    }
  }
}
