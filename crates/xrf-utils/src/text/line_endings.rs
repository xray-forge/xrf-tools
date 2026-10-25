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

  /// Detects the more frequent LF or CRLF ending, choosing LF on a tie.
  pub fn detect(content: &[u8]) -> Option<Self> {
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

    Some(if crlf > lf { Self::Crlf } else { Self::Lf })
  }

  /// Converts CRLF and bare CR sequences to LF.
  ///
  /// Used when comparing two renderings of the same content without treating host-specific line endings as a difference.
  pub fn normalize(content: &str) -> String {
    content.replace("\r\n", "\n").replace('\r', "\n")
  }

  /// Rewrites every line ending in `content` as this one.
  pub fn apply(self, content: &str) -> String {
    match self {
      Self::Lf => Self::normalize(content),
      Self::Crlf => Self::normalize(content).replace('\n', "\r\n"),
    }
  }
}

impl FromStr for LineEndings {
  type Err = XrfError;

  /// Parses the case-sensitive names `lf` and `crlf`.
  ///
  /// # Errors
  ///
  /// Returns an error for any other spelling, including uppercase names and `cr`.
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

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::LineEndings;

  #[test]
  fn parses_the_two_spellings_and_refuses_everything_else() {
    assert_eq!(LineEndings::from_str("lf").unwrap(), LineEndings::Lf);
    assert_eq!(LineEndings::from_str("crlf").unwrap(), LineEndings::Crlf);
    assert!(LineEndings::from_str("LF").is_err());
    assert!(LineEndings::from_str("cr").is_err());
  }

  #[test]
  fn normalizes_every_spelling_to_lf() {
    assert_eq!(LineEndings::normalize("a\r\nb\rc\nd"), "a\nb\nc\nd");
  }

  #[test]
  fn applies_endings_over_whatever_was_there() {
    assert_eq!(LineEndings::Lf.apply("a\r\nb\nc"), "a\nb\nc");
    assert_eq!(LineEndings::Crlf.apply("a\r\nb\nc"), "a\r\nb\r\nc");
  }

  #[test]
  fn detects_the_dominant_ending() {
    assert_eq!(LineEndings::detect(b"a\nb\nc"), Some(LineEndings::Lf));
    assert_eq!(LineEndings::detect(b"a\r\nb\r\nc"), Some(LineEndings::Crlf));
    // Two CRLF against one LF: the convention most of the file is written in.
    assert_eq!(LineEndings::detect(b"a\r\nb\r\nc\nd"), Some(LineEndings::Crlf));
    assert_eq!(LineEndings::detect(b"a\r\nb\nc\nd"), Some(LineEndings::Lf));
  }

  #[test]
  fn a_file_with_nothing_to_preserve_answers_nothing() {
    assert_eq!(LineEndings::detect(b""), None);
    assert_eq!(LineEndings::detect(b"{}"), None);
    // A bare CR is not a line ending this workspace writes, and on its own leaves nothing to preserve.
    assert_eq!(LineEndings::detect(b"a\rb"), None);
  }

  #[test]
  fn an_even_split_settles_on_lf() {
    assert_eq!(LineEndings::detect(b"a\r\nb\nc"), Some(LineEndings::Lf));
  }
}
