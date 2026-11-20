use xrf_level::DetailModel;

use crate::level::build::level_build_atlas_place::LevelBuildAtlasPlace;

/// A library detail object found drawing one of a level's detail models ([`LevelDetailLibrary::find`]).
///
/// [`LevelDetailLibrary::find`]: crate::level::details::level_detail_library::LevelDetailLibrary::find
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LevelDetailMatch<'a> {
  /// The object's reference under its library root.
  pub(crate) reference: &'a str,
  /// The model the object exports, its coordinates those of its own texture.
  pub(crate) source: &'a DetailModel,
  /// Where `MergedTextureRemapUV` put the texture in the level's atlas, fitted from the two models' coordinates; none
  /// where the source's coordinates span no area on an axis, so give no scale there.
  pub(crate) place: Option<LevelBuildAtlasPlace>,
}
