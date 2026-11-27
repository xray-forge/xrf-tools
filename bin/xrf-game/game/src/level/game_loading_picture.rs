use xrf_dds::DdsFile;
use xrf_error::XrfResult;
use xrf_game_developer::DeveloperPicture;
use xrf_random::{RandomAddress, RandomStream};
use xrf_vfs::XraySearch;

/// The picture a level's loading view shows, picked as X-Ray picks its loading screen's level logo
/// (`IGame_Persistent::Level_Set`): one of `intro\intro_<level>_1`, `_2` and on, at random, where the tree has them;
/// else `intro\intro_<level>`; else `intro\intro_no_start_picture`; else none.
pub(crate) struct GameLoadingPicture;

impl GameLoadingPicture {
  /// Where the pictures are, under the tree's textures.
  const DIRECTORY: &'static str = "textures\\intro\\intro_";

  /// The picture X-Ray falls back to for a level that has none.
  const NO_PICTURE: &'static str = "no_start_picture";

  const RANDOM_DOMAIN: u64 = RandomAddress::domain("game.loading.picture");

  /// The picture of `level` in `search`, the pick among several keyed by the game's `seed`; none, logged, where the
  /// tree has none or it does not decode.
  pub(crate) fn read(search: &XraySearch, level: &str, seed: u64) -> Option<DeveloperPicture> {
    let path: String = Self::pick(search, level, seed)?;

    match Self::decode(search, &path) {
      Ok(picture) => Some(picture),
      Err(error) => {
        log::warn!("The loading picture '{path}' does not decode: {error}");

        None
      }
    }
  }

  /// The picture's path: a numbered one of `level`'s at random, else its own, else the fallback.
  fn pick(search: &XraySearch, level: &str, seed: u64) -> Option<String> {
    let numbered = |number: u64| format!("{}{level}_{number}.dds", Self::DIRECTORY);
    let count: u64 = (1..)
      .take_while(|number| Self::exists(search, &numbered(*number)))
      .count() as u64;

    if count > 0 {
      let mut random: RandomStream = RandomStream::new(
        seed,
        RandomAddress {
          domain: Self::RANDOM_DOMAIN,
          ..RandomAddress::default()
        },
      );

      return Some(numbered(random.below(count) + 1));
    }

    [level, Self::NO_PICTURE]
      .into_iter()
      .map(|name| format!("{}{name}.dds", Self::DIRECTORY))
      .find(|path| Self::exists(search, path))
  }

  fn exists(search: &XraySearch, path: &str) -> bool {
    search.find(path).is_ok_and(|asset| asset.is_some())
  }

  fn decode(search: &XraySearch, path: &str) -> XrfResult<DeveloperPicture> {
    let image = DdsFile::read_from_vec(search.read(path)?)?.decode_rgba(0)?;
    let (width, height): (u32, u32) = image.dimensions();

    log::info!("Loading picture '{path}', {width}x{height}");

    Ok(DeveloperPicture {
      width,
      height,
      rgba: image.into_raw(),
    })
  }
}
