use std::sync::{Arc, Mutex, MutexGuard};

use serde::{Deserialize, Serialize};
use xrf_dds::DdsFile;
use xrf_vfs::XrayRoots;

use crate::core::types::TauriResult;
use crate::plugins::textures::catalog::TextureCatalogMode;
use crate::plugins::textures::encoding::{TextureEncodingFormat, TextureEncodingSession};

/// A browsing session the backend is holding: what is mounted, and how it was listed.
///
/// The mode travels with the roots because a reload has to come back to the listing it left. The same directory is a
/// game tree or a folder of loose files depending on nothing but this, and restoring the wrong one shows an empty
/// tree over a session that was there a moment ago.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureBrowseSession {
  pub roots: XrayRoots,
  pub mode: TextureCatalogMode,
}

/// What the textures tool is holding between calls.
pub struct TextureState {
  /// The session the explorer is browsing, or `None` when nothing is open.
  pub opened: Mutex<Option<TextureBrowseSession>>,
  /// The encodes the last format comparison produced, or `None` when none has run since the tool opened.
  ///
  /// One session rather than a map keyed by texture: the editor holds one node at a time, and a comparison is
  /// megabytes of block data that nobody wants accumulating behind a person browsing a tree. A new comparison replaces
  /// it whole, which is what makes an encode belonging to a texture nobody is looking at any more unreachable from a
  /// later save.
  pub encodings: Arc<Mutex<Option<TextureEncodingSession>>>,
}

impl TextureState {
  pub fn new() -> Self {
    Self {
      opened: Mutex::new(None),
      encodings: Arc::new(Mutex::new(None)),
    }
  }

  /// Read from one candidate of the held comparison, or say why it is not there to read.
  ///
  /// Takes the reader rather than answering the file, because the encodes are megabytes that only exist inside the
  /// lock: handing one out would mean either cloning it or keeping the session unusable for as long as the caller
  /// holds it. Every use is a read of the bytes it already has - serializing them for a save, decoding them for a
  /// picture - so the lock is held for exactly that.
  pub fn with_held_encoding<T>(
    &self,
    format: TextureEncodingFormat,
    read: impl FnOnce(&DdsFile) -> TauriResult<T>,
  ) -> TauriResult<T> {
    let held: MutexGuard<Option<TextureEncodingSession>> = self
      .encodings
      .lock()
      .map_err(|error| format!("The held texture encodings are unavailable: {error}"))?;
    let session: &TextureEncodingSession = held
      .as_ref()
      .ok_or_else(|| String::from("No encoded texture is held; compare the formats first"))?;
    let file: &DdsFile = session.get(format).ok_or_else(|| {
      format!(
        "The held comparison of '{}' does not carry that format; compare the formats again",
        session.label
      )
    })?;

    read(file)
  }
}
