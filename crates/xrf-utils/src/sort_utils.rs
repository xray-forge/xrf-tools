use std::cmp::Ordering;

/// Compare two strings the way rustfmt orders identifiers under `style_edition = "2024"`.
///
/// Plain byte order reads wrong wherever a name carries a number: it puts `item10` before `item2` and
/// `ammo-11.43x23` before `ammo-5.45x39`. This is the ordering rustfmt moved to for that reason (RFC 3424), and the one
/// the TypeScript ecosystem calls "natural" — `eslint-plugin-simple-import-sort`, `eslint-plugin-perfectionist`, and
/// ESLint's own `sort-keys` with `natural: true`.
///
/// Three rules, derived from what rustfmt actually emits rather than from the prose about it:
///
/// - Where both sides hold a digit, the whole runs of digits are compared as numbers: `a2` before `a10`.
/// - Where two runs are numerically equal but spelled differently, the **longer** one sorts first: `a000`, `a00`, `a0`
///   and `a001`, `a01`, `a1`. This is what keeps the order total, and it is not raw byte order — bytes agree on
///   `001`/`01`/`1` and invert on `000`/`00`/`0`.
/// - Everywhere else characters compare ordinarily, except that `_` sorts before everything: `b_c`, `b1c`, `bAc`.
///
/// Comparison is by character, not by chunk, so a digit meeting a non-digit falls through to the character rule and
/// `a_b` precedes `a1` — chunking those as `a_b` against `a` would make one a prefix of the other and get it backwards.
pub fn natural_cmp(left: &str, right: &str) -> Ordering {
  let mut left_at: usize = 0;
  let mut right_at: usize = 0;

  loop {
    let left_char: Option<char> = left[left_at..].chars().next();
    let right_char: Option<char> = right[right_at..].chars().next();

    let (left_char, right_char): (char, char) = match (left_char, right_char) {
      (None, None) => return Ordering::Equal,
      (None, Some(_)) => return Ordering::Less,
      (Some(_), None) => return Ordering::Greater,
      (Some(left_char), Some(right_char)) => (left_char, right_char),
    };

    if left_char.is_ascii_digit() && right_char.is_ascii_digit() {
      let left_end: usize = digit_run_end(left, left_at);
      let right_end: usize = digit_run_end(right, right_at);

      match compare_digit_runs(&left[left_at..left_end], &right[right_at..right_end]) {
        Ordering::Equal => {
          left_at = left_end;
          right_at = right_end;
        }
        ordering => return ordering,
      }

      continue;
    }

    match compare_chars(left_char, right_char) {
      Ordering::Equal => {
        left_at += left_char.len_utf8();
        right_at += right_char.len_utf8();
      }
      ordering => return ordering,
    }
  }
}

/// Where the run of digits starting at `at` ends.
fn digit_run_end(value: &str, at: usize) -> usize {
  at + value[at..]
    .find(|character: char| !character.is_ascii_digit())
    .unwrap_or(value.len() - at)
}

/// Compare two runs of digits as the numbers they spell.
///
/// Leading zeros are stripped before the comparison rather than parsed away, so an id carrying a number longer than any
/// integer type still orders correctly instead of saturating or refusing.
fn compare_digit_runs(left: &str, right: &str) -> Ordering {
  let left_digits: &str = left.trim_start_matches('0');
  let right_digits: &str = right.trim_start_matches('0');

  left_digits
    .len()
    .cmp(&right_digits.len())
    .then_with(|| left_digits.cmp(right_digits))
    // Numerically equal and spelled differently: the longer run first. Two equal-length runs of the same value are the
    // same bytes, so this leaves nothing undecided.
    .then_with(|| right.len().cmp(&left.len()))
}

/// Compare two characters, with `_` ahead of everything else.
fn compare_chars(left: char, right: char) -> Ordering {
  match (left == '_', right == '_') {
    (true, true) => Ordering::Equal,
    (true, false) => Ordering::Less,
    (false, true) => Ordering::Greater,
    (false, false) => left.cmp(&right),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn sorted(values: &[&str]) -> Vec<String> {
    let mut values: Vec<String> = values.iter().map(|value| String::from(*value)).collect();

    values.sort_by(|left, right| natural_cmp(left, right));
    values
  }

  #[test]
  fn orders_a_number_as_a_number() {
    assert_eq!(sorted(&["item10", "item2", "item1"]), ["item1", "item2", "item10"]);
  }

  #[test]
  fn matches_what_rustfmt_emits_for_a_mixed_list() {
    // Pinned against `rustfmt --edition 2024` over the same identifiers, so a change here is a deliberate divergence
    // from the ordering the workspace's own Rust sources are formatted with.
    assert_eq!(
      sorted(&[
        "Aa", "Ba", "_a", "_b", "aa", "ba", "a_b", "ab", "a1", "a01", "a001", "a2", "a10", "x1y2", "x1y10", "Zz", "zZ"
      ]),
      [
        "_a", "_b", "Aa", "Ba", "Zz", "a_b", "a001", "a01", "a1", "a2", "a10", "aa", "ab", "ba", "x1y2", "x1y10", "zZ"
      ]
    );
  }

  #[test]
  fn an_underscore_precedes_digits_uppercase_and_lowercase() {
    assert_eq!(sorted(&["bAc", "b1c", "b_c"]), ["b_c", "b1c", "bAc"]);
  }

  #[test]
  fn a_longer_spelling_of_one_number_sorts_first() {
    assert_eq!(sorted(&["a0", "a000", "a00"]), ["a000", "a00", "a0"]);
    assert_eq!(sorted(&["a1", "a001", "a01"]), ["a001", "a01", "a1"]);
  }

  #[test]
  fn a_digit_meeting_a_non_digit_falls_through_to_the_character_rule() {
    // Chunked as `a_b` against `a` these would compare as prefixes and come out the other way round.
    assert_eq!(sorted(&["a1", "a_b"]), ["a_b", "a1"]);
    assert_eq!(sorted(&["aa", "a1"]), ["a1", "aa"]);
  }

  #[test]
  fn orders_the_translation_ids_this_exists_for() {
    assert_eq!(
      sorted(&["st_thanks10", "st_thanks2", "st_thanks3"]),
      ["st_thanks2", "st_thanks3", "st_thanks10"]
    );
    assert_eq!(
      sorted(&["ammo-11.43x23-fmj", "ammo-5.45x39-ap"]),
      ["ammo-5.45x39-ap", "ammo-11.43x23-fmj"]
    );
    assert_eq!(
      sorted(&["jup_b10_ufo_searching_tips", "jup_b1_bar_door"]),
      ["jup_b1_bar_door", "jup_b10_ufo_searching_tips"]
    );
  }

  #[test]
  fn a_number_longer_than_any_integer_type_still_orders() {
    assert_eq!(
      sorted(&["a99999999999999999999999999", "a100000000000000000000000000"]),
      ["a99999999999999999999999999", "a100000000000000000000000000"]
    );
  }

  #[test]
  fn a_prefix_precedes_what_extends_it() {
    assert_eq!(sorted(&["ab", "a", "abc"]), ["a", "ab", "abc"]);
  }

  #[test]
  fn non_ascii_text_compares_without_panicking_on_a_char_boundary() {
    assert_eq!(sorted(&["ыb", "ыa"]), ["ыa", "ыb"]);
    assert_eq!(sorted(&["ы2", "ы10"]), ["ы2", "ы10"]);
  }

  #[test]
  fn equal_strings_are_equal() {
    assert_eq!(natural_cmp("st_a", "st_a"), Ordering::Equal);
    assert_eq!(natural_cmp("", ""), Ordering::Equal);
  }
}
