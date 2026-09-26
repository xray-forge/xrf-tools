//! How the engine reads a field's text as a value, `CInifile::r_bool` and `r_float`.

/// The words `CInifile::r_bool` takes as true, in any case; anything else is false.
const TRUE_WORDS: [&str; 4] = ["on", "yes", "true", "1"];

/// `CInifile::r_bool`: one of the true words, in any case and trimmed, or false.
pub(crate) fn read_engine_bool(value: &str) -> bool {
  let value: &str = value.trim();

  TRUE_WORDS.iter().any(|it| value.eq_ignore_ascii_case(it))
}

/// `CInifile::r_float`, through `atof`: the longest leading decimal number, sign and exponent included, or zero.
pub(crate) fn read_engine_float(value: &str) -> f32 {
  let text: &str = value.trim_start();
  let bytes: &[u8] = text.as_bytes();
  let digits = |from: usize| -> usize { bytes[from..].iter().take_while(|it| it.is_ascii_digit()).count() };
  let mut end: usize = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
  let whole: usize = digits(end);

  end += whole;

  let fraction: usize = if bytes.get(end) == Some(&b'.') {
    digits(end + 1)
  } else {
    0
  };

  if whole + fraction == 0 {
    return 0.0;
  }

  if bytes.get(end) == Some(&b'.') {
    end += 1 + fraction;
  }

  // An exponent counts only with a digit after it, as `strtod` takes it.
  if matches!(bytes.get(end), Some(b'e' | b'E')) {
    let sign: usize = usize::from(matches!(bytes.get(end + 1), Some(b'+' | b'-')));
    let exponent: usize = digits(end + 1 + sign);

    if exponent > 0 {
      end += 1 + sign + exponent;
    }
  }

  text[..end].parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
  use crate::syntax::ltx_value::{read_engine_bool, read_engine_float};

  #[test]
  fn reads_a_flag_as_r_bool_does() {
    assert!(read_engine_bool("On"));
    assert!(read_engine_bool(" 1 "));
    assert!(!read_engine_bool("no"));
    assert!(!read_engine_bool("truer"));
  }

  #[test]
  fn reads_a_number_as_atof_does() {
    assert_eq!(read_engine_float("2.5"), 2.5);
    assert_eq!(read_engine_float(" -0.25 ; a comment the reader kept"), -0.25);
    assert_eq!(read_engine_float("1e2x"), 100.0);
    assert_eq!(read_engine_float("3e"), 3.0);
    assert_eq!(read_engine_float("3."), 3.0);
    assert_eq!(read_engine_float(".5"), 0.5);
    assert_eq!(read_engine_float("-."), 0.0);
    assert_eq!(read_engine_float("abc"), 0.0);
  }
}
