use serde::Serialize;
use xrf_texture::ImageShape;

/// What a picture the webview renders itself turns out to be.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDescriptor {
  /// Absent when the bytes carry no header this reads, which is a picture worth neither drawing nor measuring.
  pub shape: Option<ImageShape>,
  /// What to hand the bytes over as, taken from the extension rather than from the content.
  pub media_type: String,
}
