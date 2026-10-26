use xrf_utils::XRayEncoding;

/// What a code page can hold, asked by the build, both writers and edit validation of a language's own.
pub(crate) struct TranslationCodePage;

impl TranslationCodePage {
  /// The first character `encoding` cannot represent, or none when it holds the whole value.
  pub(crate) fn find_unencodable(value: &str, encoding: XRayEncoding) -> Option<char> {
    // One pass over the value first, because almost every value is writable and needs no per-character search.
    if !encoding.encode(value).2 {
      return None;
    }

    value
      .chars()
      .find(|character| encoding.encode(&String::from(*character)).2)
  }

  /// How a refusal names a character: itself and its code point, since the one a code page lacks is often one a
  /// reader cannot see.
  pub(crate) fn describe(character: char) -> String {
    format!("'{character}' (U+{:04X})", character as u32)
  }
}
