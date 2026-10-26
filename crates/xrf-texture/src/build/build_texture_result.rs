use std::path::PathBuf;

use crate::build::recipe::build_texture_omission::BuildTextureOmission;
use crate::bump::GenerateBumpResult;

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
  /// Not a failure: a surface should say what was left out without calling it a problem.
  pub omissions: Vec<BuildTextureOmission>,
  /// The pair a bump descriptor built, its companion and its gloss verdict; none for any other descriptor.
  pub bump: Option<GenerateBumpResult>,
}
