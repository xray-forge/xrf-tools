use std::path::PathBuf;

use xrf_audio::{AudioAcoustics, AudioEngineSettings, AudioEvent, AudioOutputChoice, AudioServer};
use xrf_error::XrfResult;
use xrf_level_source::LevelAudio;

use crate::directories::game_resource::GameResource;

/// The game's sound: the audio server a shown level plays through, on the system's default output, its acoustics on
/// Steam Audio where the library is found, dry where it is not.
pub struct GameSound;

impl GameSound {
  /// Where the game finds Steam Audio.
  const ACOUSTICS: GameResource = GameResource {
    packaged: "audio/steam-audio",
    checkout: concat!(env!("CARGO_MANIFEST_DIR"), "/../../../.sdks/steam-audio"),
  };

  /// Starts the audio server. With no output device it plays to nothing rather than failing.
  ///
  /// # Errors
  ///
  /// Returns why the server's threads cannot be started.
  pub fn start() -> XrfResult<LevelAudio> {
    // A folder holding no Steam Audio loads the acoustics dry.
    let acoustics: PathBuf = Self::ACOUSTICS
      .locate()
      .unwrap_or_else(|| PathBuf::from(Self::ACOUSTICS.checkout));
    let server: AudioServer = AudioServer::start(
      AudioEngineSettings::default(),
      AudioOutputChoice::default(),
      AudioAcoustics::load(&acoustics),
      |_: AudioEvent| {},
    )?;

    Ok(LevelAudio::new(server))
  }
}
