use std::any::type_name;
use std::str::FromStr;

use xrf_error::{XrfError, XrfResult};

use crate::syntax::read_engine_bool;

/// Reads fields of the one section an [`crate::LtxRead`] found.
///
/// A key written twice reads as the last value it was given.
pub trait LtxSectionRead {
  /// The value of `key`, if the section has the field.
  fn get(&self, key: &str) -> Option<&str>;

  /// The value as `CInifile::r_bool` reads it; `None` for a key the section lacks.
  fn get_bool(&self, key: &str) -> Option<bool> {
    self.get(key).map(read_engine_bool)
  }

  /// The value of a field a record requires, parsed as `T` strictly, which is how an exported record is read back.
  ///
  /// # Errors
  ///
  /// Returns a parsing error when the section lacks the field, or its value is not a valid `T`.
  fn read_field<T: FromStr>(&self, key: &str) -> XrfResult<T> {
    let value: &str = self
      .get(key)
      .ok_or_else(|| XrfError::new_parsing_error(format!("Field '{key}' was not found in ltx file")))?;

    value.parse::<T>().map_err(|_| {
      XrfError::new_parsing_error(format!(
        "Failed to parse ltx field '{key}' value '{value}', valid {} is expected",
        type_name::<T>(),
      ))
    })
  }

  /// The value of a field a record may leave out, parsed as `T` strictly; `None` for a key the section lacks.
  ///
  /// # Errors
  ///
  /// Returns a parsing error when the value is not a valid `T`.
  fn read_optional_field<T: FromStr>(&self, key: &str) -> XrfResult<Option<T>> {
    self
      .get(key)
      .map(|value| {
        value.parse::<T>().map_err(|_| {
          XrfError::new_parsing_error(format!(
            "Failed to parse optional ltx field '{key}' value '{value}', valid {} is expected",
            type_name::<T>()
          ))
        })
      })
      .transpose()
  }
}
