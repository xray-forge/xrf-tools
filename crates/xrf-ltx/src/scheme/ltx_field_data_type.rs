use std::fmt::Display;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::assert_equal;

use crate::scheme::TupleSeparator;
use crate::syntax::{LTX_SYMBOL_ARRAY, LTX_SYMBOL_OPTIONAL};

#[derive(Clone, Debug, PartialEq)]
pub enum LtxFieldDataType {
  TypeAny,
  TypeBool,
  TypeCondlist,
  TypeConst(String),
  TypeEnum(Vec<String>),
  TypeF32,
  TypeI16,
  TypeI32,
  TypeI8,
  TypeRgb,
  TypeRgba,
  TypeSection,
  TypeString,
  TypeTuple(Vec<LtxFieldDataType>, Vec<String>, TupleSeparator),
  TypeU16,
  TypeU32,
  TypeU8,
  TypeUnknown,
  TypeVector,
}

impl LtxFieldDataType {
  /// Parse data type enum variant from provided string option.
  pub fn from_field_data(field_name: &str, section_name: &str, data: &str) -> XrfResult<Self> {
    let mut data: &str = data;

    // Respect optionals.
    if data.starts_with(LTX_SYMBOL_OPTIONAL) {
      data = &data[1..];
    }

    // Respect arrays.
    if data.ends_with(LTX_SYMBOL_ARRAY) {
      data = &data[0..(data.len() - 2)];
    }

    Ok(match data {
      "bool" => Self::TypeBool,
      "condlist" => Self::TypeCondlist,
      "f32" => Self::TypeF32,
      "i16" => Self::TypeI16,
      "i32" => Self::TypeI32,
      "i8" => Self::TypeI8,
      "rgb" => Self::TypeRgb,
      "rgba" => Self::TypeRgba,
      "section" => Self::TypeSection,
      "string" => Self::TypeString,
      "u16" => Self::TypeU16,
      "u32" => Self::TypeU32,
      "u8" => Self::TypeU8,
      "vector" => Self::TypeVector,
      field_type => {
        if field_type.starts_with("enum") {
          Self::parse_enum(field_name, section_name, data)?
        } else if field_type.starts_with("tuple") {
          Self::parse_tuple(field_name, section_name, data)?
        } else if field_type.starts_with("const") {
          Self::parse_const(field_name, section_name, data)?
        } else {
          Self::TypeUnknown
        }
      }
    })
  }

  /// Parse data array type from value.
  pub fn is_field_data_array(data: &str) -> bool {
    data.ends_with(LTX_SYMBOL_ARRAY)
  }

  /// Parse data optional type from value.
  pub fn is_field_data_optional(data: &str) -> bool {
    data.starts_with(LTX_SYMBOL_OPTIONAL)
  }
}

impl LtxFieldDataType {
  fn parse_enum(field_name: &str, section_name: &str, value: &str) -> XrfResult<LtxFieldDataType> {
    let mut allowed_values: Vec<String> = Vec::new();

    match value.split_once(':') {
      None => {
        return Err(XrfError::new_read_error(format!(
          "Failed to read scheme enum type for field '{section_name}', expected ':' separated type and values"
        )));
      }
      Some((_, allowed_values_string)) => {
        for allowed in allowed_values_string.trim().split(',').filter_map(|it| {
          let trimmed: &str = it.trim();

          if trimmed.is_empty() { None } else { Some(trimmed) }
        }) {
          allowed_values.push(allowed.into());
        }
      }
    }

    if allowed_values.is_empty() {
      Err(XrfError::new_ltx_scheme_error(
        section_name,
        field_name,
        "Failed to parse enum type, expected comma separated list of possible values after 'enum:'",
      ))
    } else {
      Ok(Self::TypeEnum(allowed_values))
    }
  }

  fn parse_const(field_name: &str, section_name: &str, value: &str) -> XrfResult<LtxFieldDataType> {
    match value.split_once(':') {
      None => Err(XrfError::new_read_error(format!(
        "Failed to read scheme const type for field '{section_name}', expected ':' prepended value"
      ))),
      Some((_, const_value)) => {
        let const_value: &str = const_value.trim();

        if const_value.is_empty() {
          Err(XrfError::new_ltx_scheme_error(
            section_name,
            field_name,
            "Failed to parse const type, expected actual data after 'const:'",
          ))
        } else {
          Ok(Self::TypeConst(const_value.into()))
        }
      }
    }
  }

  fn parse_tuple(field_name: &str, section_name: &str, value: &str) -> XrfResult<LtxFieldDataType> {
    let mut types: Vec<LtxFieldDataType> = Vec::new();
    let mut types_raw: Vec<String> = Vec::new();

    let (separator, allowed_values_string): (TupleSeparator, &str) = match value.split_once(':') {
      None => {
        return Err(XrfError::new_read_error(format!(
          "Failed to read scheme tuple type for field '{section_name}', expected ':' separated types"
        )));
      }
      Some((tuple_type, allowed_values_string)) => {
        let separator: TupleSeparator = TupleSeparator::from_tuple_type(field_name, section_name, tuple_type)?;

        (separator, allowed_values_string)
      }
    };

    for tuple_entry_raw in Self::split_tuple_entries(field_name, section_name, allowed_values_string)? {
      let (tuple_entry, tuple_entry_raw): (&str, String) =
        Self::parse_tuple_entry(field_name, section_name, tuple_entry_raw)?;

      let tuple_entry: XrfResult<LtxFieldDataType> = Self::from_field_data(field_name, section_name, tuple_entry);

      match tuple_entry? {
        Self::TypeTuple(_, _, _) => {
          return Err(XrfError::new_read_error(format!(
            "Failed to read scheme for field '{section_name}', tuple cannot contain nested tuples"
          )));
        }
        schema => {
          types.push(schema);
          types_raw.push(tuple_entry_raw);
        }
      }
    }

    if types.is_empty() {
      Err(XrfError::new_ltx_scheme_error(
        section_name,
        field_name,
        "Failed to parse tuple type, expected comma separated list of possible values after 'tuple:'",
      ))
    } else {
      assert_equal(
        types_raw.len(),
        types.len(),
        "Expected same count of raw and converted types for tuple type definition",
      )?;

      Ok(Self::TypeTuple(types, types_raw, separator))
    }
  }

  fn split_tuple_entries<'a>(field_name: &str, section_name: &str, value: &'a str) -> XrfResult<Vec<&'a str>> {
    let mut entries: Vec<&str> = Vec::new();
    let mut start: usize = 0;
    let mut depth: usize = 0;

    for (index, character) in value.char_indices() {
      match character {
        '(' => depth += 1,
        ')' => {
          if depth == 0 {
            return Err(XrfError::new_ltx_scheme_error(
              section_name,
              field_name,
              "Failed to parse tuple type, unexpected closing parenthesis",
            ));
          }

          depth -= 1;
        }
        ',' if depth == 0 => {
          entries.push(&value[start..index]);
          start = index + character.len_utf8();
        }
        _ => {}
      }
    }

    if depth != 0 {
      return Err(XrfError::new_ltx_scheme_error(
        section_name,
        field_name,
        "Failed to parse tuple type, expected closing parenthesis",
      ));
    }

    entries.push(&value[start..]);

    Ok(entries)
  }

  fn parse_tuple_entry<'a>(field_name: &str, section_name: &str, value: &'a str) -> XrfResult<(&'a str, String)> {
    let value: &str = value.trim();
    let is_grouped: bool = value.starts_with('(') && value.ends_with(')');
    let inner_value: &str = if is_grouped {
      value[1..(value.len() - 1)].trim()
    } else {
      value
    };

    if inner_value.is_empty() {
      return Err(XrfError::new_ltx_scheme_error(
        section_name,
        field_name,
        "Failed to parse tuple type, expected a field type entry",
      ));
    }

    if !is_grouped && inner_value.trim_start_matches('?').starts_with("enum") {
      return Err(XrfError::new_ltx_scheme_error(
        section_name,
        field_name,
        "Failed to parse tuple type, enum entries must be enclosed in parentheses",
      ));
    }

    Ok((inner_value, String::from(value)))
  }

  /// Parse data type enum variant from provided string option.
  pub fn from_field_data_optional(
    field_name: &str,
    section_name: &str,
    data: Option<&str>,
  ) -> XrfResult<LtxFieldDataType> {
    data.map_or(Ok(Self::TypeAny), |data| {
      Self::from_field_data(field_name, section_name, data)
    })
  }
}

impl Display for LtxFieldDataType {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let type_string: String = match self {
      Self::TypeAny => String::from("any"),
      Self::TypeBool => String::from("bool"),
      Self::TypeCondlist => String::from("condlist"),
      Self::TypeConst(value) => format!("const:{value}"),
      Self::TypeEnum(values) => format!("enum:{}", values.join(",")),
      Self::TypeF32 => String::from("g43"),
      Self::TypeI16 => String::from("i16"),
      Self::TypeI32 => String::from("i32"),
      Self::TypeI8 => String::from("i8"),
      Self::TypeRgb => String::from("rgb"),
      Self::TypeRgba => String::from("rgba"),
      Self::TypeSection => String::from("section"),
      Self::TypeString => String::from("string"),
      Self::TypeTuple(_, raw_types, separator) => match separator {
        TupleSeparator::Comma => format!("tuple:{}", raw_types.join(",")),
        TupleSeparator::Pipe => format!("tuple@pipe:{}", raw_types.join(",")),
      },
      Self::TypeU16 => String::from("u16"),
      Self::TypeU32 => String::from("u32"),
      Self::TypeU8 => String::from("u8"),
      Self::TypeUnknown => String::from("unknown"),
      Self::TypeVector => String::from("vector"),
    };

    write!(formatter, "{}", type_string)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::scheme::TupleSeparator;

  #[test]
  fn parses_pipe_tuple_with_grouped_enum() {
    let data_type: LtxFieldDataType =
      LtxFieldDataType::from_field_data("on_signal", "$scheme_common", "tuple@pipe:(enum:hit,miss),condlist")
        .expect("Expected pipe tuple type to parse");

    assert_eq!(
      data_type,
      LtxFieldDataType::TypeTuple(
        vec![
          LtxFieldDataType::TypeEnum(vec![String::from("hit"), String::from("miss")]),
          LtxFieldDataType::TypeCondlist,
        ],
        vec![String::from("(enum:hit,miss)"), String::from("condlist")],
        TupleSeparator::Pipe,
      )
    );
  }

  #[test]
  fn rejects_ungrouped_enum_in_tuple() {
    let error: XrfError =
      LtxFieldDataType::from_field_data("on_signal", "$scheme_common", "tuple@pipe:enum:hit,condlist")
        .expect_err("Expected ungrouped enum tuple entry to fail");

    assert!(
      error
        .to_string()
        .contains("enum entries must be enclosed in parentheses")
    );
  }

  #[test]
  fn supports_explicit_comma_tuple_separator() {
    let data_type: LtxFieldDataType = LtxFieldDataType::from_field_data("value", "section", "tuple@comma:string,u32")
      .expect("Expected comma tuple type to parse");

    assert_eq!(data_type.to_string(), "tuple:string,u32");
  }
}
