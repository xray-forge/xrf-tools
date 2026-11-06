use std::fmt::{self, Display, Write};
use std::str;

/// A value's text as [`Display`] writes it, held on the stack while it fits.
///
/// The text is copied once more into the field that keeps it, so formatting through `to_string` first paid a second,
/// short-lived allocation per field: about a sixth of building a spawn's graph points, 2.3 million of them. A value
/// longer than the stack space, such as a script's custom data, moves to the heap as `to_string` would.
pub(crate) struct SectionValueText {
  stack: [u8; Self::STACK_BYTES],
  length: usize,
  heap: Option<String>,
}

impl SectionValueText {
  /// Room for any number or vector a record writes.
  const STACK_BYTES: usize = 64;

  pub(crate) fn of<V: Display>(value: V) -> Self {
    let mut text: Self = Self {
      stack: [0; Self::STACK_BYTES],
      length: 0,
      heap: None,
    };

    // Writing to memory fails only when the value's own `Display` reports an error, which `to_string` panics on.
    write!(text, "{value}").expect("a Display implementation returned an error unexpectedly");

    text
  }

  pub(crate) fn as_str(&self) -> &str {
    match &self.heap {
      Some(heap) => heap,
      // Only whole `&str` pieces are ever copied in, so the bytes are valid UTF-8 up to `length`.
      None => str::from_utf8(&self.stack[..self.length]).expect("whole pieces of text to stay valid UTF-8"),
    }
  }
}

impl Write for SectionValueText {
  fn write_str(&mut self, piece: &str) -> fmt::Result {
    if let Some(heap) = &mut self.heap {
      heap.push_str(piece);

      return Ok(());
    }

    let end: usize = self.length + piece.len();

    if end <= Self::STACK_BYTES {
      self.stack[self.length..end].copy_from_slice(piece.as_bytes());
      self.length = end;
    } else {
      let mut heap: String = String::with_capacity(end);

      heap.push_str(self.as_str());
      heap.push_str(piece);
      self.heap = Some(heap);
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use crate::ltx::section::section_value_text::SectionValueText;

  #[test]
  fn writes_what_to_string_writes_on_the_stack_or_past_it() {
    let long: String = "ж".repeat(40);

    for (value, expected) in [
      (SectionValueText::of(12.599999_f32), String::from("12.599999")),
      (SectionValueText::of(u32::MAX), String::from("4294967295")),
      (SectionValueText::of(""), String::new()),
      (SectionValueText::of("x".repeat(64)), "x".repeat(64)),
      (SectionValueText::of("x".repeat(65)), "x".repeat(65)),
      (SectionValueText::of(&long), long.clone()),
      (
        SectionValueText::of(format_args!("{}{}", "a".repeat(60), "bbbbbbbb")),
        format!("{}bbbbbbbb", "a".repeat(60)),
      ),
    ] {
      assert_eq!(value.as_str(), expected);
    }
  }

  /// What a record exports is numbers, so their edges are pinned against `to_string`, which the setter's callers used to
  /// write: signed zero, the infinities, NaN, the smallest subnormal and the largest values, whose text passes the stack.
  #[test]
  fn writes_every_edge_of_a_number_as_to_string_does() {
    let floats: [f32; 12] = [
      0.0,
      -0.0,
      f32::NAN,
      -f32::NAN,
      f32::INFINITY,
      f32::NEG_INFINITY,
      f32::MIN_POSITIVE,
      f32::from_bits(1),
      f32::MAX,
      f32::MIN,
      0.1,
      -1.0e-30,
    ];

    for value in floats {
      assert_eq!(SectionValueText::of(value).as_str(), value.to_string(), "{value:?}");
      assert_eq!(
        SectionValueText::of(f64::from(value)).as_str(),
        f64::from(value).to_string()
      );
    }

    for value in [f64::MAX, f64::MIN, f64::from_bits(1), f64::MIN_POSITIVE] {
      assert_eq!(SectionValueText::of(value).as_str(), value.to_string(), "{value:?}");
    }

    assert_eq!(SectionValueText::of(i64::MIN).as_str(), i64::MIN.to_string());
    assert_eq!(SectionValueText::of(u64::MAX).as_str(), u64::MAX.to_string());
    assert_eq!(SectionValueText::of(u128::MAX).as_str(), u128::MAX.to_string());
    assert_eq!(SectionValueText::of(true).as_str(), "true");
    assert_eq!(SectionValueText::of('ж').as_str(), "ж");
  }
}
