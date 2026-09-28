//! How the engine reads a field's text as a value, `CInifile::r_bool` and `r_float`.

/// The words `CInifile::r_bool` takes as true, in any case; anything else is false.
const TRUE_WORDS: [&str; 4] = ["on", "yes", "true", "1"];

/// `CInifile::r_bool`: one of the true words, in any case and trimmed, or false.
pub fn read_engine_bool(value: &str) -> bool {
  let value: &str = value.trim();

  TRUE_WORDS.iter().any(|it| value.eq_ignore_ascii_case(it))
}

/// `CInifile::r_float`, through `atof`: the longest leading decimal number, sign and exponent included, or zero.
pub fn read_engine_float(value: &str) -> f32 {
  scan_engine_float(value).map_or(0.0, |(number, _)| number)
}

/// `strtod` as `atof` and `sscanf`'s `%f` read a number: leading whitespace skipped, then the longest decimal number,
/// sign and exponent included. Answers the number and how many bytes it took, whitespace counted; none where the text
/// does not start with a number, which `atof` reads as zero and `sscanf` stops at.
pub fn scan_engine_float(value: &str) -> Option<(f32, usize)> {
  let text: &str = value.trim_start();
  let skipped: usize = value.len() - text.len();
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
    return None;
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

  Some((text[..end].parse().unwrap_or(0.0), skipped + end))
}

/// `sscanf(value, "%f,%f,...")` with `most` conversions, as `r_fvector2`, `r_fvector3` and `r_fvector4` read a vector:
/// each number must be followed straight by a comma for the next to be read, and the scan stops at the first that is
/// not. Answers the components it read, fewer than `most` where it stopped early; the engine keeps the rest at zero.
pub fn read_engine_floats(value: &str, most: usize) -> Vec<f32> {
  let mut components: Vec<f32> = Vec::with_capacity(most);
  let mut rest: &str = value;

  while components.len() < most {
    let Some((number, used)) = scan_engine_float(rest) else {
      break;
    };

    components.push(number);
    rest = &rest[used..];

    match rest.strip_prefix(',') {
      Some(next) => rest = next,
      None => break,
    }
  }

  components
}

/// `CInifile::r_s32`, through `atoi`: the longest leading integer, sign included, or zero.
pub fn read_engine_integer(value: &str) -> i32 {
  let text: &str = value.trim_start();
  let bytes: &[u8] = text.as_bytes();
  let sign: usize = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
  let digits: usize = bytes[sign..].iter().take_while(|it| it.is_ascii_digit()).count();

  if digits == 0 {
    return 0;
  }

  // `atoi` overflows as the platform does; saturating is the one reading that is never worse.
  text[..sign + digits].parse::<i64>().map_or(0, |number| number.clamp(i32::MIN.into(), i32::MAX.into()) as i32)
}

#[cfg(test)]
mod tests {
  use crate::syntax::ltx_value::{read_engine_bool, read_engine_float, read_engine_floats, read_engine_integer};

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

  #[test]
  fn reads_a_vector_as_sscanf_does() {
    assert_eq!(read_engine_floats("0.1, 0.2, 0.3", 3), vec![0.1, 0.2, 0.3]);
    assert_eq!(read_engine_floats("1,2,3,4,5", 3), vec![1.0, 2.0, 3.0]);
    assert_eq!(read_engine_floats("1, 2", 3), vec![1.0, 2.0]);
    // A space before the comma stops the scan: the format's comma has to match the very next character.
    assert_eq!(read_engine_floats("1 ,2", 2), vec![1.0]);
    assert_eq!(read_engine_floats("1,x,3", 3), vec![1.0]);
    assert!(read_engine_floats("", 3).is_empty());
  }

  #[test]
  fn reads_an_integer_as_atoi_does() {
    assert_eq!(read_engine_integer(" 25000"), 25000);
    assert_eq!(read_engine_integer("-3.7"), -3);
    assert_eq!(read_engine_integer("x1"), 0);
  }
}
