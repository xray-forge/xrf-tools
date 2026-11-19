use std::str::FromStr;

use xrf_error::{XrfError, XrfResult};

use crate::ini_field::IniField;

/// One `[name]` section and its fields, in the order they were read or added, each key once.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct IniSection {
  pub name: String,
  /// Changed only through [`Self::set`] and [`Self::remove`], which keep each key once.
  pub(crate) fields: Vec<IniField>,
}

impl IniSection {
  pub fn new(name: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      fields: Vec::new(),
    }
  }

  pub fn fields(&self) -> &[IniField] {
    &self.fields
  }

  pub fn get(&self, key: &str) -> Option<&str> {
    self
      .fields
      .iter()
      .find(|field| field.key == key)
      .map(|field| field.value.as_str())
  }

  /// The value of a field the section must hold.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section and key when the field is missing.
  pub fn get_required(&self, key: &str) -> XrfResult<&str> {
    self
      .get(key)
      .ok_or_else(|| XrfError::new_parsing_error(format!("Section [{}] has no field '{key}'", self.name)))
  }

  /// A field the section must hold, parsed whole as `T`.
  ///
  /// # Errors
  ///
  /// Returns an error naming the section and key when the field is missing or does not parse.
  pub fn get_parsed<T: FromStr>(&self, key: &str) -> XrfResult<T> {
    let value: &str = self.get_required(key)?;

    value.parse::<T>().map_err(|_| {
      XrfError::new_parsing_error(format!(
        "Field [{}] {key} = '{value}' is not a {}",
        self.name,
        std::any::type_name::<T>()
      ))
    })
  }

  /// Sets a field in place, or adds it after the others.
  pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
    let field: IniField = IniField::new(key, value);

    match self.fields.iter_mut().find(|existing| existing.key == field.key) {
      Some(existing) => existing.value = field.value,
      None => self.fields.push(field),
    }
  }

  /// Removes a field and returns its value.
  pub fn remove(&mut self, key: &str) -> Option<String> {
    let index: usize = self.fields.iter().position(|field| field.key == key)?;

    Some(self.fields.remove(index).value)
  }
}
