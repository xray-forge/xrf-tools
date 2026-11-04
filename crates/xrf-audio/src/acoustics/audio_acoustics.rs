use std::path::Path;
use std::sync::Arc;

#[cfg(feature = "steam-audio")]
use crate::acoustics::steam::steam_audio::SteamAudio;

/// Whether the audio's acoustics run, and through what: Steam Audio, loaded from the directory the host names, places
/// every voice, muffles it through the scene's walls and gives the scene's rooms their reverb. Where it is not built in,
/// its library does not load or is not the version this build binds, sound plays dry: one distance curve and panning,
/// with no occlusion or reverb, and the reason is kept for the report. A clone shares the same library.
#[derive(Clone)]
pub struct AudioAcoustics {
  state: Arc<AcousticsState>,
}

enum AcousticsState {
  Dry(String),
  #[cfg(feature = "steam-audio")]
  Steam(SteamAudio),
}

impl AudioAcoustics {
  /// The library's file name on this platform.
  pub const LIBRARY: &'static str = if cfg!(windows) {
    "phonon.dll"
  } else if cfg!(target_os = "macos") {
    "libphonon.dylib"
  } else {
    "libphonon.so"
  };

  /// Steam Audio loaded from `directory`, which holds [`Self::LIBRARY`]; dry, and why, where it cannot be.
  pub fn load(directory: &Path) -> Self {
    #[cfg(feature = "steam-audio")]
    let state: AcousticsState = match SteamAudio::load(directory) {
      Ok(steam) => {
        log::info!("Acoustics run on Steam Audio from {}", directory.display());

        AcousticsState::Steam(steam)
      }
      Err(reason) => {
        log::warn!("Sound plays dry: {reason}");

        AcousticsState::Dry(reason)
      }
    };

    #[cfg(not(feature = "steam-audio"))]
    let state: AcousticsState = {
      let _ = directory;

      AcousticsState::Dry(String::from("This build has no Steam Audio"))
    };

    Self { state: Arc::new(state) }
  }

  /// Dry, for `reason`.
  pub fn dry(reason: impl Into<String>) -> Self {
    Self {
      state: Arc::new(AcousticsState::Dry(reason.into())),
    }
  }

  pub fn is_running(&self) -> bool {
    self.get_failure().is_none()
  }

  /// Why sound plays dry; none while the acoustics run.
  pub fn get_failure(&self) -> Option<&str> {
    match self.state.as_ref() {
      AcousticsState::Dry(reason) => Some(reason),
      #[cfg(feature = "steam-audio")]
      AcousticsState::Steam(_) => None,
    }
  }

  #[cfg(feature = "steam-audio")]
  pub(crate) fn get_steam(&self) -> Option<&SteamAudio> {
    match self.state.as_ref() {
      AcousticsState::Dry(_) => None,
      AcousticsState::Steam(steam) => Some(steam),
    }
  }
}

impl Default for AudioAcoustics {
  fn default() -> Self {
    Self::dry("No acoustics were asked for")
  }
}

impl std::fmt::Debug for AudioAcoustics {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.get_failure() {
      Some(reason) => write!(formatter, "AudioAcoustics::Dry({reason:?})"),
      None => write!(formatter, "AudioAcoustics::Steam"),
    }
  }
}
