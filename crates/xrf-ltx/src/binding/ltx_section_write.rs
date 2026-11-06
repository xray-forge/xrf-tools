use std::fmt::Display;

/// Sets fields of the one section an [`crate::LtxWrite`] opened.
pub trait LtxSectionWrite {
  /// Set a field to the text `value` displays as.
  ///
  /// Takes anything that displays, text as much as a number or a vector, so a value is written as `to_string` would
  /// write it without the copy `to_string` makes; lend a value that is not `Copy` (`&self.name`).
  fn set<K, V>(&mut self, key: K, value: V) -> &mut Self
  where
    K: AsRef<str>,
    V: Display;

  /// Set a field when there is a value for it, as [`Self::set`] does; `None` writes nothing.
  fn set_optional<K, V>(&mut self, key: K, value: Option<V>) -> &mut Self
  where
    K: AsRef<str>,
    V: Display,
  {
    if let Some(value) = value {
      self.set(key, value);
    }

    self
  }
}
