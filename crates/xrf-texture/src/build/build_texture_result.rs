use std::path::PathBuf;

use crate::build::build_texture_omission::BuildTextureOmission;

/// What a rebuilt texture came to.
#[derive(Clone, Debug, PartialEq)]
pub struct BuildTextureResult {
  pub destination: PathBuf,
  /// Size of the source, which the descriptor's own width and height are refreshed from.
  pub width: u32,
  pub height: u32,
  /// Levels written, counting the base.
  pub mipmap_levels: u32,
  /// Recipe fields the descriptor asked for and the build did not carry out.
  ///
  /// Not a failure. Better than half the corpus asks for colour dithering alone, so a build reporting nothing here is
  /// the exception rather than the rule, and a surface should say what was left out without calling it a problem.
  pub omissions: Vec<BuildTextureOmission>,
}
