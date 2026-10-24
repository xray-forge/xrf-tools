use serde::{Deserialize, Serialize};
use xrf_app_core::assets::FileStamp;
use xrf_app_core::session::SessionId;
use xrf_texture::TextureEncodingFormat;

/// The texture half of a save: one candidate a held comparison encoded, written over the texture that comparison was
/// made of, which it names itself.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureEncodingSave {
  /// The comparison holding the candidate.
  pub session_id: SessionId,
  pub format: TextureEncodingFormat,
  /// The stamp the editor read on the texture, or `None` where it saw none.
  pub expected: Option<FileStamp>,
}
