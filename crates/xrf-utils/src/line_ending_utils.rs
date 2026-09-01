use std::str::FromStr;

use xrf_error::{XrfError, XrfResult};

/// Physical line endings used for generated and rewritten text artifacts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LineEndings {
  Lf,
  Crlf,
}

impl LineEndings {
  /// The sequence this ending writes.
  #[inline]
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Lf => "\n",
      Self::Crlf => "\r\n",
    }
  }
}

impl FromStr for LineEndings {
  type Err = XrfError;

  fn from_str(value: &str) -> XrfResult<Self> {
    match value {
      "lf" => Ok(Self::Lf),
      "crlf" => Ok(Self::Crlf),
      _ => Err(XrfError::new_invalid_error(format!(
        "Unsupported line endings '{value}'. Expected lf or crlf."
      ))),
    }
  }
}

/// Converts CRLF and bare CR sequences to LF.
///
/// Used when comparing two renderings of the same content without treating host-specific line endings as a difference.
pub fn normalize_line_endings(content: &str) -> String {
  content.replace("\r\n", "\n").replace('\r', "\n")
}

/// Rewrites every line ending in `content` as `line_endings`.
pub fn apply_line_endings(content: &str, line_endings: LineEndings) -> String {
  match line_endings {
    LineEndings::Lf => normalize_line_endings(content),
    LineEndings::Crlf => normalize_line_endings(content).replace('\n', "\r\n"),
  }
}

/// The line ending an existing file already uses, or nothing when it holds no line break at all.
///
/// The dominant ending wins rather than the first one seen, so a file somebody edited with two tools keeps the
/// convention most of it is written in instead of whichever line happens to come first. A file with no break to read is
/// `None`, which leaves the choice to the caller — there is nothing in the file to preserve.
///
/// Counts CRLF as CRLF rather than as a CR plus an LF, so the two are genuinely compared. A tie goes to LF, because a
/// file split evenly between the two has no convention to preserve and LF is what every writer here produced before
/// this existed.
pub fn detect_line_endings(content: &[u8]) -> Option<LineEndings> {
  let mut lf: usize = 0;
  let mut crlf: usize = 0;

  for (index, byte) in content.iter().enumerate() {
    if *byte != b'\n' {
      continue;
    }

    if index > 0 && content[index - 1] == b'\r' {
      crlf += 1;
    } else {
      lf += 1;
    }
  }

  if lf == 0 && crlf == 0 {
    return None;
  }

  Some(if crlf > lf { LineEndings::Crlf } else { LineEndings::Lf })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_the_two_spellings_and_refuses_everything_else() {
    assert_eq!(LineEndings::from_str("lf").unwrap(), LineEndings::Lf);
    assert_eq!(LineEndings::from_str("crlf").unwrap(), LineEndings::Crlf);
    assert!(LineEndings::from_str("LF").is_err());
    assert!(LineEndings::from_str("cr").is_err());
  }

  #[test]
  fn normalizes_every_spelling_to_lf() {
    assert_eq!(normalize_line_endings("a\r\nb\rc\nd"), "a\nb\nc\nd");
  }

  #[test]
  fn applies_endings_over_whatever_was_there() {
    assert_eq!(apply_line_endings("a\r\nb\nc", LineEndings::Lf), "a\nb\nc");
    assert_eq!(apply_line_endings("a\r\nb\nc", LineEndings::Crlf), "a\r\nb\r\nc");
  }

  #[test]
  fn detects_the_dominant_ending() {
    assert_eq!(detect_line_endings(b"a\nb\nc"), Some(LineEndings::Lf));
    assert_eq!(detect_line_endings(b"a\r\nb\r\nc"), Some(LineEndings::Crlf));
    // Two CRLF against one LF: the convention most of the file is written in.
    assert_eq!(detect_line_endings(b"a\r\nb\r\nc\nd"), Some(LineEndings::Crlf));
    assert_eq!(detect_line_endings(b"a\r\nb\nc\nd"), Some(LineEndings::Lf));
  }

  #[test]
  fn a_file_with_nothing_to_preserve_answers_nothing() {
    assert_eq!(detect_line_endings(b""), None);
    assert_eq!(detect_line_endings(b"{}"), None);
    // A bare CR is not a line ending this workspace writes, and on its own leaves nothing to preserve.
    assert_eq!(detect_line_endings(b"a\rb"), None);
  }

  #[test]
  fn an_even_split_settles_on_lf() {
    assert_eq!(detect_line_endings(b"a\r\nb\nc"), Some(LineEndings::Lf));
  }
}
