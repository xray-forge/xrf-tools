/// A fixed-width format field a length can be narrowed into.
pub trait FormatWidth: Sized + TryFrom<usize> {
  const LABEL: &'static str;
}

impl FormatWidth for u8 {
  const LABEL: &'static str = "u8";
}

impl FormatWidth for u16 {
  const LABEL: &'static str = "u16";
}

impl FormatWidth for u32 {
  const LABEL: &'static str = "u32";
}
