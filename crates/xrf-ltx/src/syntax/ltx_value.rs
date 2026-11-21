//! How the engine reads a field's text as a value: `CInifile::Load`'s comments and quotes, `r_bool` and `r_float`.

use xrf_error::XrfError;

use crate::syntax::ltx_syntax::LTX_SYMBOL_COMMENT;

/// The words `CInifile::r_bool` takes as true, in any case; anything else is false.
const TRUE_WORDS: [&str; 4] = ["on", "yes", "true", "1"];

/// What `CInifile::Load` keeps of `line` before reading a field: everything before the first `;`, or the first `/`
/// when a second follows it, unless that cut falls between the line's first two quotes.
///
/// Only the first `/` counts, so `a/b // c` keeps its `// c`, as in the engine.
pub(crate) fn strip_engine_comment(line: &str) -> &str {
  let semicolon: Option<usize> = line.find(LTX_SYMBOL_COMMENT);
  let slashes: Option<usize> = line.find('/').filter(|&slash| line[slash + 1..].starts_with('/'));
  let Some(start) = semicolon.into_iter().chain(slashes).min() else {
    return line;
  };
  let is_quoted: bool = line[..start]
    .find('"')
    .is_some_and(|open| line[open + 1..].find('"').is_some_and(|close| open + 1 + close > start));

  if is_quoted { line } else { &line[..start] }
}

/// Whether `value`, as the engine keeps it, holds an odd number of quotes: `CInifile::Load` then reads the lines after
/// it into it, so a reader refuses such a value in a section, and a writer refuses to write one there.
pub fn leaves_quote_open(value: &str) -> bool {
  strip_engine_comment(value).bytes().filter(|&byte| byte == b'"').count() % 2 == 1
}

/// The error a writer returns instead of writing a value in a section that [`leaves_quote_open`].
pub(crate) fn new_open_quote_error(section: &str, key: &str) -> XrfError {
  XrfError::new_convert_error(format!(
    "Value of '{key}' in section '{section}' leaves a quote open, which no reader takes back"
  ))
}

/// Whether `value`, written as a field's value, reads back as itself: no padding the reader trims, no line break, no
/// comment the reader cuts and no quote left open.
pub fn is_value_kept(value: &str) -> bool {
  value == value.trim()
    && !value.contains(['\r', '\n'])
    && strip_engine_comment(value) == value
    && !leaves_quote_open(value)
}

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

#[cfg(test)]
mod tests {
  use crate::syntax::{is_value_kept, leaves_quote_open, read_engine_bool, read_engine_float, read_engine_floats};

  #[test]
  fn tells_a_value_whose_quote_stays_open_where_the_engine_keeps_it() {
    assert!(leaves_quote_open("\"a"));
    assert!(leaves_quote_open("\"a\" \"b"));
    assert!(!leaves_quote_open("\"a\""));
    // The engine cuts at the `;` after the first pair, so the third quote is never read.
    assert!(!leaves_quote_open("\"a\" ; \"b"));
    assert!(!leaves_quote_open("plain"));
  }

  #[test]
  fn tells_a_value_that_reads_back_as_written() {
    assert!(is_value_kept("wp00|a={+gar_conc_camp_arbaiten_3}probe_way"));
    assert!(is_value_kept("\"a;b\""));
    assert!(!is_value_kept("wp00 "), "trimmed");
    assert!(!is_value_kept("a ; b"), "cut at the comment");
    assert!(!is_value_kept("a // b"), "cut at the comment");
    assert!(!is_value_kept("\"a"), "a quote left open");
    assert!(!is_value_kept("a\nb"), "a line break");
  }

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
}
