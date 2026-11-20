use serde::Serialize;
use xrf_texture::GenerateBumpFormat;

/// One format a bump pair can be generated in, with what a choice between them needs to show.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextureBumpFormatEntry {
  pub format: GenerateBumpFormat,
  /// The format's name with the DXT name the SDK uses for it.
  pub label: String,
  /// What the renderers together make of a pair in this format, as a sentence a badge can show.
  pub support_summary: String,
  /// Whether a renderer is known to refuse the pair, so such a choice is flagged before it is written.
  pub is_refused_anywhere: bool,
}

impl TextureBumpFormatEntry {
  /// Every format a pair can be generated in, the SDK's own first.
  pub(crate) fn of_all() -> Vec<Self> {
    GenerateBumpFormat::ALL
      .into_iter()
      .map(|format| Self {
        format,
        label: format.to_candidate().label().to_owned(),
        support_summary: format.to_candidate().support_summary(),
        is_refused_anywhere: format.to_candidate().is_refused_anywhere(),
      })
      .collect()
  }
}
