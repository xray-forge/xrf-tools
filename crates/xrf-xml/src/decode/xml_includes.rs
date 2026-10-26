use xrf_error::{XrfError, XrfResult};
use xrf_utils::{XRayEncoding, decode_bytes_to_string_without_bom_handling, new_utf8_encoder};

use crate::bytes::find_bytes;
use crate::decode::XmlEncoding;

/// The engine's `#include "<name>"` lines, which splice other files into a document before it is parsed.
pub struct XmlIncludes;

impl XmlIncludes {
  /// The directive a line opens with to splice another file in its place.
  const DIRECTIVE: &'static [u8] = b"#include";

  /// How deep includes nest before the engine gives up on a document.
  const MAX_DEPTH: usize = 128;

  /// The longest include name the engine accepts.
  const MAX_NAME_LENGTH: usize = 1024;

  const UTF8_BYTE_ORDER_MARK: &'static [u8] = &[0xEF, 0xBB, 0xBF];
  const DECLARATION_START: &'static [u8] = b"<?xml";
  const DECLARATION_END: &'static [u8] = b"?>";

  /// Splice every `#include "<name>"` line into a document, the way the engine's XML loader does before parsing it.
  ///
  /// The engine reads a document line by line: a line opening with `#include` after blanks is replaced by the named
  /// file, itself expanded the same way, and every other line is copied. Shipped character descriptions take their
  /// supplies and most of their dialogs this way, so reading one without expanding it reads a different character than
  /// the game does.
  ///
  /// `read` maps an include name to its bytes. The engine opens it under `$game_config$`, so a reader of a game tree
  /// maps `gameplay\character_dialogs.xml` to `configs\gameplay\character_dialogs.xml`. Bytes are spliced as they are,
  /// like the engine does, so the including document's encoding decodes the result; it decodes every include name too,
  /// nested ones as well, since their files' declarations are dropped where they are spliced.
  ///
  /// # Errors
  ///
  /// Returns a parsing error for a malformed directive, a name its document's encoding does not decode, or includes
  /// nested past the engine's 128 levels; an encoding error for a declaration naming no known code page; and what `read`
  /// returns for a name it cannot open. The engine refuses the loaders' documents on each.
  pub fn expand(source: &[u8], read: &mut dyn FnMut(&str) -> XrfResult<Vec<u8>>) -> XrfResult<Vec<u8>> {
    // Nearly every document has no include at all, and it is then returned as written.
    if find_bytes(source, Self::DIRECTIVE).is_none() {
      return Ok(source.to_vec());
    }

    let encoding: XRayEncoding = XmlEncoding::find_declared(source)?.unwrap_or_else(new_utf8_encoder);
    let mut expanded: Vec<u8> = Vec::with_capacity(source.len());

    Self::expand_into(source, encoding, read, 0, &mut expanded)?;

    Ok(expanded)
  }

  /// The names a document's `#include` lines ask for, in order, without reading them.
  ///
  /// A malformed directive, or a name the document's encoding does not decode, is left out, since [`Self::expand`]
  /// reports it when the document is read.
  pub fn list(source: &[u8]) -> Vec<String> {
    if find_bytes(source, Self::DIRECTIVE).is_none() {
      return Vec::new();
    }

    let encoding: XRayEncoding = XmlEncoding::find_declared(source)
      .ok()
      .flatten()
      .unwrap_or_else(new_utf8_encoder);

    source
      .split_inclusive(|byte| *byte == b'\n')
      .filter_map(|line| Self::parse_include(line.trim_ascii_end(), encoding).ok().flatten())
      .collect()
  }

  fn expand_into(
    source: &[u8],
    encoding: XRayEncoding,
    read: &mut dyn FnMut(&str) -> XrfResult<Vec<u8>>,
    depth: usize,
    expanded: &mut Vec<u8>,
  ) -> XrfResult {
    if depth >= Self::MAX_DEPTH {
      return Err(XrfError::new_parsing_error(format!(
        "XML includes nest deeper than {} levels",
        Self::MAX_DEPTH
      )));
    }

    for line in source.split_inclusive(|byte| *byte == b'\n') {
      let Some(name) = Self::parse_include(line.trim_ascii_end(), encoding)? else {
        expanded.extend_from_slice(line);

        continue;
      };

      Self::expand_into(Self::strip_prolog(&read(&name)?), encoding, read, depth + 1, expanded)?;

      // The engine writes every line it copies with a line break, the included file's last one too.
      if !expanded.ends_with(b"\n") {
        expanded.extend_from_slice(b"\r\n");
      }
    }

    Ok(())
  }

  /// The name a line includes, decoded as its document is, or `None` for a line that is not an include.
  ///
  /// Follows `ParseInclude` in `xrCore/XML/XMLDocument.cpp`: blanks may precede the directive and the opening quote,
  /// and a name missing its closing quote runs to the end of the line.
  fn parse_include(line: &[u8], encoding: XRayEncoding) -> XrfResult<Option<String>> {
    let line: &[u8] = Self::trim_blanks(line);

    let Some(rest) = line.strip_prefix(Self::DIRECTIVE) else {
      return Ok(None);
    };

    let Some(quoted) = Self::trim_blanks(rest).strip_prefix(b"\"") else {
      return Err(XrfError::new_parsing_error(format!(
        "Invalid XML include directive '{}', the name must be quoted",
        String::from_utf8_lossy(line)
      )));
    };

    let name: &[u8] = quoted.split(|byte| *byte == b'"').next().unwrap_or_default();

    if name.is_empty() || name.len() > Self::MAX_NAME_LENGTH {
      return Err(XrfError::new_parsing_error(format!(
        "Invalid XML include directive '{}', the name must be 1 to {} bytes",
        String::from_utf8_lossy(line),
        Self::MAX_NAME_LENGTH
      )));
    }

    decode_bytes_to_string_without_bom_handling(name, encoding)
      .map(Some)
      .map_err(|_| {
        XrfError::new_parsing_error(format!(
          "Invalid XML include directive '{}', the name is not {}",
          String::from_utf8_lossy(line),
          encoding.name()
        ))
      })
  }

  /// An included file's content without its byte order mark and XML declaration.
  ///
  /// The engine splices both in mid-document, where its parser skips a declaration and a strict one refuses it, so
  /// dropping them reads the document the engine reads. Anomaly's character parts carry declarations of their own.
  fn strip_prolog(content: &[u8]) -> &[u8] {
    let content: &[u8] = content.strip_prefix(Self::UTF8_BYTE_ORDER_MARK).unwrap_or(content);
    let declaration: &[u8] = content.trim_ascii_start();

    if !declaration.starts_with(Self::DECLARATION_START) {
      return content;
    }

    match find_bytes(declaration, Self::DECLARATION_END) {
      Some(end) => &declaration[end + Self::DECLARATION_END.len()..],
      None => content,
    }
  }

  /// Skip leading spaces and tabs, the blanks `std::isblank` matches.
  fn trim_blanks(bytes: &[u8]) -> &[u8] {
    let start: usize = bytes
      .iter()
      .position(|byte| *byte != b' ' && *byte != b'\t')
      .unwrap_or(bytes.len());

    &bytes[start..]
  }
}

#[cfg(test)]
mod tests;
