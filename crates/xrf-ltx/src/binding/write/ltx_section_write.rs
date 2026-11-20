use std::fmt::Display;

/// Sets fields of the one section an [`crate::LtxWrite`] opened.
pub trait LtxSectionWrite {
  /// Sets a field to the text `value` displays as, without the copy `to_string` makes.
  ///
  /// Lend a value that is not `Copy` (`&self.name`).
  fn set<K, V>(&mut self, key: K, value: V) -> &mut Self
  where
    K: AsRef<str>,
    V: Display;

  /// Sets a field as [`Self::set`] does; `None` writes nothing.
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
