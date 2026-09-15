use std::fmt::Display;
use std::str::FromStr;

use xrf_error::{XrfError, XrfResult};

/// Stringify provided error, to simplify casting at a boundary that carries messages rather than errors.
#[inline]
pub fn error_to_string<T: Display>(error: T) -> String {
  error.to_string()
}

/// Stringify provided vector as comma-separated values.
#[inline]
pub fn vector_to_string<T: ToString>(slice: &[T]) -> String {
  vector_to_string_sep(slice, ",")
}

/// Stringify provided vector as pattern-separated values.
#[inline]
pub fn vector_to_string_sep<T: ToString>(slice: &[T], sep: &str) -> String {
  slice.iter().map(T::to_string).collect::<Vec<_>>().join(sep)
}

/// Parse vector of values from string.
#[inline]
pub fn vector_from_string<T: FromStr>(string: &str) -> XrfResult<Vec<T>> {
  vector_from_string_sep(string, ",")
}

/// Parse vector of values from string with separator.
#[inline]
pub fn vector_from_string_sep<T: FromStr>(string: &str, sep: &str) -> XrfResult<Vec<T>> {
  let source: &str = string.trim();

  if source.is_empty() {
    return Ok(vec![]);
  }

  let mut vector: Vec<T> = Vec::new();

  for it in source.split(sep) {
    vector.push(match T::from_str(it.trim()) {
      Ok(value) => value,
      _ => {
        return Err(XrfError::new_parsing_error(format!(
          "Failed to parse vector from string value \"{}\"",
          it
        )));
      }
    });
  }

  Ok(vector)
}

/// Read vector of values from serialized by comma string.
#[inline]
pub fn vector_from_string_sized<T: FromStr>(string: &str, size: usize) -> XrfResult<Vec<T>> {
  vector_from_string_sep_sized(string, ",", size)
}

/// Read vector of values from serialized by separator string.
#[inline]
pub fn vector_from_string_sep_sized<T: FromStr>(string: &str, sep: &str, size: usize) -> XrfResult<Vec<T>> {
  let vector: Vec<T> = vector_from_string_sep(string, sep)?;

  if vector.len() == size {
    Ok(vector)
  } else {
    Err(XrfError::new_parsing_error(format!(
      "Failed to parse sized vector from string, not matching size ({} instead of {})",
      vector.len(),
      size
    )))
  }
}

/// Convert a snake or kebab case identifier to camel case.
///
/// Separators are dropped and the character after each is upper cased; nothing else is touched, so an already camel case
/// string survives unchanged. Used where a wire name has to become a JavaScript identifier.
pub fn to_camel_case(value: &str) -> String {
  let mut camel: String = String::with_capacity(value.len());
  let mut capitalize_next: bool = false;

  for character in value.chars() {
    match character {
      '_' | '-' => capitalize_next = true,
      _ if capitalize_next => {
        camel.extend(character.to_uppercase());
        capitalize_next = false;
      }
      _ => camel.push(character),
    }
  }

  camel
}

#[cfg(test)]
mod tests {
  use xrf_error::XrfResult;

  use crate::{
    to_camel_case, vector_from_string, vector_from_string_sep, vector_from_string_sep_sized, vector_from_string_sized,
    vector_to_string, vector_to_string_sep,
  };

  #[test]
  fn test_to_camel_case() {
    assert_eq!(to_camel_case("read_geometry"), "readGeometry");
    assert_eq!(to_camel_case("equipment-icons"), "equipmentIcons");
    assert_eq!(to_camel_case("open"), "open");
    assert_eq!(to_camel_case("alreadyCamel"), "alreadyCamel");
    assert_eq!(to_camel_case("read_ogf_file"), "readOgfFile");
    assert_eq!(to_camel_case(""), "");
    // A trailing separator has nothing to raise, and must not panic or leave the separator behind.
    assert_eq!(to_camel_case("trailing_"), "trailing");
    assert_eq!(to_camel_case("__leading"), "Leading");
  }

  #[test]
  fn test_vector_to_string() -> XrfResult {
    assert_eq!(vector_to_string(&[1, 2, 3, 4]), "1,2,3,4");
    assert_eq!(vector_to_string(&["ab", "cd", "ef", "gh"]), "ab,cd,ef,gh");

    Ok(())
  }

  #[test]
  fn test_vector_to_string_sep() -> XrfResult {
    assert_eq!(vector_to_string_sep(&[1, 2, 3, 4], "#"), "1#2#3#4");
    assert_eq!(vector_to_string_sep(&["ab", "cd", "ef", "gh"], "$"), "ab$cd$ef$gh");

    Ok(())
  }

  #[test]
  fn test_vector_from_string() -> XrfResult {
    assert_eq!(vector_from_string::<i32>("1,2,3,4")?, [1, 2, 3, 4]);
    assert_eq!(
      vector_from_string::<f32>("1,2.5,-33,4.0")?,
      [1f32, 2.5f32, -33f32, 4f32]
    );
    assert_eq!(vector_from_string::<String>("a,b,c")?, ["a", "b", "c",]);

    Ok(())
  }

  #[test]
  fn test_vector_from_string_sep() -> XrfResult {
    assert_eq!(vector_from_string_sep::<i32>("1#2#3#4", "#")?, [1, 2, 3, 4]);
    assert_eq!(
      vector_from_string_sep::<f32>("1$2.5$-33$4.0", "$")?,
      [1f32, 2.5f32, -33f32, 4f32]
    );
    assert_eq!(vector_from_string_sep::<String>("a#$b#$c", "#$")?, ["a", "b", "c",]);

    Ok(())
  }

  #[test]
  fn test_vector_from_string_sized() -> XrfResult {
    assert_eq!(vector_from_string_sized::<i32>("1,2,3,4", 4)?, [1, 2, 3, 4]);
    assert_eq!(
      vector_from_string_sized::<f32>("1,2.5,-33,4.0,76", 5)?,
      [1f32, 2.5f32, -33f32, 4f32, 76f32]
    );
    assert_eq!(vector_from_string_sized::<String>("a,b,c", 3)?, ["a", "b", "c",]);
    assert_eq!(vector_from_string_sized::<String>("", 0)?, Vec::<String>::new());

    assert_eq!(
      vector_from_string_sized::<String>("", 3).unwrap_err().to_string(),
      "Parsing error: Failed to parse sized vector from string, not matching size (0 instead of 3)"
    );
    assert_eq!(
      vector_from_string_sized::<String>("a,b", 3).unwrap_err().to_string(),
      "Parsing error: Failed to parse sized vector from string, not matching size (2 instead of 3)"
    );

    Ok(())
  }

  #[test]
  fn test_vector_from_string_sep_sized() -> XrfResult {
    assert_eq!(vector_from_string_sep_sized::<i32>("1#2#3#4", "#", 4)?, [1, 2, 3, 4]);
    assert_eq!(
      vector_from_string_sep_sized::<f32>("1$2.5$-33$4.0$45", "$", 5)?,
      [1f32, 2.5f32, -33f32, 4f32, 45f32]
    );
    assert_eq!(
      vector_from_string_sep_sized::<String>("a#$b#$c", "#$", 3)?,
      ["a", "b", "c",]
    );
    assert_eq!(
      vector_from_string_sep_sized::<String>("", "#$", 0)?,
      Vec::<String>::new()
    );

    assert_eq!(
      vector_from_string_sep_sized::<String>("", "#", 3)
        .unwrap_err()
        .to_string(),
      "Parsing error: Failed to parse sized vector from string, not matching size (0 instead of 3)"
    );
    assert_eq!(
      vector_from_string_sep_sized::<String>("a#$b", "#$", 3)
        .unwrap_err()
        .to_string(),
      "Parsing error: Failed to parse sized vector from string, not matching size (2 instead of 3)"
    );

    Ok(())
  }
}
