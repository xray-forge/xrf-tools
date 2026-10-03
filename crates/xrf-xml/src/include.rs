use xrf_error::{XrfError, XrfResult};

/// The directive a line opens with to splice another file in its place.
const INCLUDE_DIRECTIVE: &[u8] = b"#include";

/// How deep includes nest before the engine gives up on a document.
const MAX_INCLUDE_DEPTH: usize = 128;

/// The longest include name the engine accepts.
const MAX_INCLUDE_NAME_LENGTH: usize = 1024;

const UTF8_BYTE_ORDER_MARK: &[u8] = &[0xEF, 0xBB, 0xBF];
const XML_DECLARATION_START: &[u8] = b"<?xml";
const XML_DECLARATION_END: &[u8] = b"?>";

/// Splice every `#include "<name>"` line into a document, the way the engine's XML loader does before parsing it.
///
/// The engine reads a document line by line: a line opening with `#include` after blanks is replaced by the named file,
/// itself expanded the same way, and every other line is copied. Shipped character descriptions take their supplies and
/// most of their dialogs this way, so reading one without expanding it reads a different character than the game does.
///
/// `read` maps an include name to its bytes. The engine opens it under `$game_config$`, so a reader of a game tree maps
/// `gameplay\character_dialogs.xml` to `configs\gameplay\character_dialogs.xml`. Bytes are spliced as they are, like the
/// engine does, so the including document's encoding decodes the result.
///
/// # Errors
///
/// Returns a parsing error for a malformed directive or for includes nested past the engine's limit, and what `read`
/// returns for a name it cannot open. The engine refuses the loaders' documents on each.
pub fn expand_xml_includes(source: &[u8], read: &mut dyn FnMut(&str) -> XrfResult<Vec<u8>>) -> XrfResult<Vec<u8>> {
  // Nearly every document has no include at all, and it is then returned as written.
  if !contains(source, INCLUDE_DIRECTIVE) {
    return Ok(source.to_vec());
  }

  let mut expanded: Vec<u8> = Vec::with_capacity(source.len());

  expand_into(source, read, 0, &mut expanded)?;

  Ok(expanded)
}

/// The names a document's `#include` lines ask for, in order, without reading them.
///
/// A malformed directive is left out, since `expand_xml_includes` reports it when the document is read.
pub fn list_xml_includes(source: &[u8]) -> Vec<String> {
  if !contains(source, INCLUDE_DIRECTIVE) {
    return Vec::new();
  }

  source
    .split_inclusive(|byte| *byte == b'\n')
    .filter_map(|line| parse_include(line.trim_ascii_end()).ok().flatten())
    .collect()
}

fn expand_into(
  source: &[u8],
  read: &mut dyn FnMut(&str) -> XrfResult<Vec<u8>>,
  depth: usize,
  expanded: &mut Vec<u8>,
) -> XrfResult {
  if depth >= MAX_INCLUDE_DEPTH {
    return Err(XrfError::new_parsing_error(format!(
      "XML includes nest deeper than {MAX_INCLUDE_DEPTH} levels"
    )));
  }

  for line in source.split_inclusive(|byte| *byte == b'\n') {
    let Some(name) = parse_include(line.trim_ascii_end())? else {
      expanded.extend_from_slice(line);

      continue;
    };

    expand_into(strip_prolog(&read(&name)?), read, depth + 1, expanded)?;

    // The engine writes every line it copies with a line break, the included file's last one too.
    if !expanded.ends_with(b"\n") {
      expanded.extend_from_slice(b"\r\n");
    }
  }

  Ok(())
}

/// The name a line includes, or `None` for a line that is not an include.
///
/// Follows `ParseInclude` in `xrCore/XML/XMLDocument.cpp`: blanks may precede the directive and the opening quote, and a
/// name missing its closing quote runs to the end of the line.
fn parse_include(line: &[u8]) -> XrfResult<Option<String>> {
  let line: &[u8] = trim_blanks(line);

  let Some(rest) = line.strip_prefix(INCLUDE_DIRECTIVE) else {
    return Ok(None);
  };

  let Some(quoted) = trim_blanks(rest).strip_prefix(b"\"") else {
    return Err(XrfError::new_parsing_error(format!(
      "Invalid XML include directive '{}', the name must be quoted",
      String::from_utf8_lossy(line)
    )));
  };

  let name: &[u8] = quoted.split(|byte| *byte == b'"').next().unwrap_or_default();

  if name.is_empty() || name.len() > MAX_INCLUDE_NAME_LENGTH {
    return Err(XrfError::new_parsing_error(format!(
      "Invalid XML include directive '{}', the name must be 1 to {MAX_INCLUDE_NAME_LENGTH} bytes",
      String::from_utf8_lossy(line)
    )));
  }

  Ok(Some(String::from_utf8_lossy(name).into_owned()))
}

/// An included file's content without its byte order mark and XML declaration.
///
/// The engine splices both in mid-document, where its parser skips a declaration and a strict one refuses it, so
/// dropping them reads the document the engine reads. Anomaly's character parts carry declarations of their own.
fn strip_prolog(content: &[u8]) -> &[u8] {
  let content: &[u8] = content.strip_prefix(UTF8_BYTE_ORDER_MARK).unwrap_or(content);
  let start: usize = content
    .iter()
    .position(|byte| !byte.is_ascii_whitespace())
    .unwrap_or(content.len());

  if !content[start..].starts_with(XML_DECLARATION_START) {
    return content;
  }

  match find(&content[start..], XML_DECLARATION_END) {
    Some(end) => &content[start + end + XML_DECLARATION_END.len()..],
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

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
  find(haystack, needle).is_some()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack.windows(needle.len()).position(|window| window == needle)
}

#[cfg(test)]
mod tests;
