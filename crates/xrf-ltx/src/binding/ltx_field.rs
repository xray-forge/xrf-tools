use std::str::FromStr;

use xrf_error::{XrfError, XrfResult};

use crate::Section;

/// Read value from ltx section and parse it as provided T type.
pub fn read_ltx_field<T: FromStr>(field_name: &str, section: &Section) -> XrfResult<T> {
  let value: &str = section
    .get(field_name)
    .ok_or_else(|| XrfError::new_parsing_error(format!("Field '{field_name}' was not found in ltx file")))?;

  Ok(match T::from_str(value) {
    Ok(value) => value,
    _ => {
      return Err(XrfError::new_parsing_error(format!(
        "Failed to parse ltx field '{}' value '{}', valid {} is expected",
        field_name,
        value,
        std::any::type_name::<T>(),
      )));
    }
  })
}

/// Read optional value from ltx section and parse it as provided T type.
pub fn read_ltx_optional_field<T: FromStr>(field_name: &str, section: &Section) -> XrfResult<Option<T>> {
  let field_data: Option<&str> = section.get(field_name);

  Ok(match field_data {
    Some(value) => match value.parse::<T>() {
      Ok(parsed) => Some(parsed),
      _ => {
        return Err(XrfError::new_parsing_error(format!(
          "Failed to parse optional ltx field '{}' value, correct {:?} is expected",
          field_name,
          std::any::type_name::<T>()
        )));
      }
    },
    None => None,
  })
}
