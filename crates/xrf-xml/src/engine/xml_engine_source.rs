use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, decode_bytes_to_string_without_bom_handling, new_utf8_encoder, new_windows1251_encoder};

use crate::decode::{XmlEncoding, XmlIncludes};
use crate::source::XmlSourceDocument;

/// A game tree's XML read the way the engine's loader reads it (`xrCore/XML/XMLDocument.cpp`): includes spliced in,
/// decoded as gameplay configs are, and parsed past the comment banners and bare ampersands its reader accepts.
pub struct XmlEngineSource;

impl XmlEngineSource {
  /// The logical directory the engine opens an include under, `$game_config$`.
  const INCLUDES_DIRECTORY: &'static str = "configs";

  const UTF8_BYTE_ORDER_MARK: &'static [u8] = &[0xEF, 0xBB, 0xBF];
  const UTF16_BYTE_ORDER_MARKS: [&'static [u8]; 2] = [&[0xFF, 0xFE], &[0xFE, 0xFF]];

  /// The logical path the engine opens an include name at: `gameplay\x.xml` is `configs\gameplay\x.xml`.
  pub fn resolve_include(name: &str) -> String {
    format!("{}\\{name}", Self::INCLUDES_DIRECTORY)
  }

  /// Every file the given documents include, as the logical path [`Self::resolve_include`] gives it.
  pub fn list_included<'a>(sources: impl IntoIterator<Item = &'a [u8]>) -> Vec<String> {
    sources
      .into_iter()
      .flat_map(XmlIncludes::list)
      .map(|name| Self::resolve_include(&name))
      .collect()
  }

  /// Read one document: its includes spliced in through `read`, which maps a logical path to its bytes, then decoded
  /// and parsed.
  ///
  /// # Errors
  ///
  /// Returns what [`XmlIncludes::expand`] and `read` return for an include, an encoding error as
  /// [`Self::find_encoding`] and [`Self::decode`] give one, and a parsing error when neither the document nor its
  /// repaired copy is well-formed or it declares a DTD ([`XmlSourceDocument::parse`]).
  pub fn parse(source: &[u8], read: &mut dyn FnMut(&str) -> XrfResult<Vec<u8>>) -> XrfResult<XmlSourceDocument> {
    let expanded: Vec<u8> = XmlIncludes::expand(source, &mut |name| read(&Self::resolve_include(name)))?;

    XmlSourceDocument::parse(Self::decode(&expanded)?)
  }

  /// The encoding a config's bytes are written in.
  ///
  /// A UTF-8 byte order mark decides it and outranks the declaration, since shipped files contradict themselves: a
  /// UTF-8 marked file declaring windows-1251 exists in reference gamedata. Without one the declaration decides, and
  /// without that windows-1251, which the engine assumes for gameplay configs.
  ///
  /// # Errors
  ///
  /// Returns an encoding error for UTF-16, which no gameplay config encoder holds and which decoding as a code page
  /// would silently mangle, and for a declaration naming a code page there is no encoder for.
  pub fn find_encoding(data: &[u8]) -> XrfResult<XRayEncoding> {
    if Self::UTF16_BYTE_ORDER_MARKS.iter().any(|mark| data.starts_with(mark)) {
      return Err(XrfError::new_encoding_error(
        "XML is UTF-16, which gameplay configs cannot be written as",
      ));
    }

    if data.starts_with(Self::UTF8_BYTE_ORDER_MARK) {
      return Ok(new_utf8_encoder());
    }

    Ok(XmlEncoding::find_declared(data)?.unwrap_or_else(new_windows1251_encoder))
  }

  /// Split a leading UTF-8 byte order mark off a config's bytes, so a rewrite can put it back as it was.
  pub fn split_byte_order_mark(data: &[u8]) -> (&[u8], &[u8]) {
    match data.strip_prefix(Self::UTF8_BYTE_ORDER_MARK) {
      Some(body) => data.split_at(data.len() - body.len()),
      None => (&[], data),
    }
  }

  /// A config's bytes as text, in the encoding [`Self::find_encoding`] finds, without its byte order mark.
  ///
  /// # Errors
  ///
  /// As [`Self::find_encoding`], and an encoding error for bytes that do not decode as it.
  pub fn decode(data: &[u8]) -> XrfResult<String> {
    let encoding: XRayEncoding = Self::find_encoding(data)?;
    let (_, body) = Self::split_byte_order_mark(data);

    // Mapped rather than converted: bytes that do not decode are the document's own defect, and `From<io::Error>` would
    // report them as an IO failure, which a caller reads as the file never having been read.
    decode_bytes_to_string_without_bom_handling(body, encoding)
      .map_err(|error| XrfError::new_encoding_error(error.to_string()))
  }
}

#[cfg(test)]
mod tests;
