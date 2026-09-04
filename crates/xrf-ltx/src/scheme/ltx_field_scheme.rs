use std::cmp;

use xrf_error::XrfError;

use crate::condlist::Condlist;
use crate::ltx::Ltx;
use crate::scheme::LtxFieldDataType;

/// Scheme definition for single field in LTX file section.
#[derive(Clone, Debug)]
pub struct LtxFieldScheme {
  pub data_type: LtxFieldDataType,
  pub is_array: bool,
  pub is_optional: bool,
  pub name: String,
  pub section: String,
  // todo: Add range (min-max) support, add fixed array len support (min-max).
  // todo: Deprecate 'strict'.
}

impl LtxFieldScheme {
  pub fn new_with_optional_type<S, F>(section: S, name: F, data_type: LtxFieldDataType) -> Self
  where
    S: Into<String>,
    F: Into<String>,
  {
    Self {
      data_type,
      is_array: false,
      is_optional: true,
      name: name.into(),
      section: section.into(),
    }
  }

  pub fn new_with_array_optional_type<S, F>(section: S, name: F, data_type: LtxFieldDataType) -> Self
  where
    S: Into<String>,
    F: Into<String>,
  {
    Self {
      data_type,
      is_array: true,
      is_optional: true,
      name: name.into(),
      section: section.into(),
    }
  }

  pub fn new_with_array_type<S, F>(section: S, name: F, data_type: LtxFieldDataType) -> Self
  where
    S: Into<String>,
    F: Into<String>,
  {
    Self {
      data_type,
      is_array: true,
      is_optional: false,
      name: name.into(),
      section: section.into(),
    }
  }

  pub fn new_with_type<S, F>(section: S, name: F, data_type: LtxFieldDataType) -> Self
  where
    S: Into<String>,
    F: Into<String>,
  {
    Self {
      data_type,
      is_array: false,
      is_optional: false,
      name: name.into(),
      section: section.into(),
    }
  }
}

impl LtxFieldScheme {
  // todo: Do not use ltx as parameter, split section check on higher level or impl 2 separate methods.
  /// Validate provided value based on current field schema definition.
  pub fn validate_value(&self, ltx: &Ltx, field_data: &str) -> Option<XrfError> {
    // Ltx-specific validation of section type.
    if self.data_type == LtxFieldDataType::TypeSection {
      if self.is_array {
        // todo: Probably merge with generic array check.
        for entry in field_data.split(',') {
          let entry: &str = entry.trim();

          if !entry.is_empty() {
            let validation_result: Option<XrfError> = self.validate_section_type_defined(ltx, entry);

            if validation_result.is_some() {
              return validation_result;
            }
          }
        }
      } else {
        return self.validate_section_type_defined(ltx, field_data);
      }

      return None;
    }

    if self.is_array {
      let array_values: Vec<&str> = field_data
        .split(',')
        .map(|it| it.trim())
        .filter(|it| !it.is_empty())
        .collect::<Vec<&str>>();

      if array_values.is_empty() && !self.is_optional {
        return Some(self.validation_error(&format!(
          "Invalid value - expected non optional array of {}",
          self.data_type,
        )));
      }

      for entry in array_values {
        let validation_result: Option<XrfError> = self.validate_data_entry_by_type(&self.data_type, entry);

        if validation_result.is_some() {
          return validation_result;
        }
      }

      None
    } else {
      self.validate_data_entry_by_type(&self.data_type, field_data)
    }
  }

  fn validate_data_entry_by_type(&self, field_type: &LtxFieldDataType, field_data: &str) -> Option<XrfError> {
    match field_type {
      LtxFieldDataType::TypeAny => None,
      LtxFieldDataType::TypeBool => self.validate_bool_type(field_data),
      LtxFieldDataType::TypeCondlist => self.validate_condlist_type(field_data),
      LtxFieldDataType::TypeConst(_) => self.validate_const(field_data),
      LtxFieldDataType::TypeEnum(_) => self.validate_enum_type(field_data),
      LtxFieldDataType::TypeF32 => self.validate_f32_type(field_data),
      LtxFieldDataType::TypeI16 => self.validate_i16_type(field_data),
      LtxFieldDataType::TypeI32 => self.validate_i32_type(field_data),
      LtxFieldDataType::TypeI8 => self.validate_i8_type(field_data),
      LtxFieldDataType::TypeRgb => self.validate_rgb_type(field_data),
      LtxFieldDataType::TypeRgba => self.validate_rgba_type(field_data),
      LtxFieldDataType::TypeSection => self.validate_section_type(field_data),
      LtxFieldDataType::TypeString => self.validate_string_type(field_data),
      LtxFieldDataType::TypeTuple(_, _, _) => self.validate_tuple_type(field_data),
      LtxFieldDataType::TypeU16 => self.validate_u16_type(field_data),
      LtxFieldDataType::TypeU32 => self.validate_u32_type(field_data),
      LtxFieldDataType::TypeU8 => self.validate_u8_type(field_data),
      LtxFieldDataType::TypeUnknown => None,
      LtxFieldDataType::TypeVector => self.validate_vector_type(field_data),
    }
  }

  fn validation_error(&self, message: &str) -> XrfError {
    XrfError::new_ltx_scheme_error(&self.section, &self.name, message)
  }
}

impl LtxFieldScheme {
  // Whether field is defined as section.
  pub fn is_section(&self) -> bool {
    self.data_type == LtxFieldDataType::TypeSection
  }
}

impl LtxFieldScheme {
  fn validate_f32_type(&self, value: &str) -> Option<XrfError> {
    match value.parse::<f32>() {
      Ok(_) => None,
      Err(_) => Some(self.validation_error(&format!(
        "Invalid value, floating point number is expected, got '{value}'"
      ))),
    }
  }

  fn validate_u32_type(&self, value: &str) -> Option<XrfError> {
    match value.parse::<u32>() {
      Ok(_) => None,
      Err(_) => Some(self.validation_error(&format!(
        "Invalid value, unsigned 32 bit number is expected, got '{value}'"
      ))),
    }
  }

  fn validate_i32_type(&self, value: &str) -> Option<XrfError> {
    match value.parse::<i32>() {
      Ok(_) => None,
      Err(_) => Some(self.validation_error(&format!(
        "Invalid value, signed 32 bit number is expected, got '{value}'"
      ))),
    }
  }

  fn validate_u16_type(&self, value: &str) -> Option<XrfError> {
    match value.parse::<u16>() {
      Ok(_) => None,
      Err(_) => Some(self.validation_error(&format!(
        "Invalid value, unsigned 16 bit number is expected, got '{value}'"
      ))),
    }
  }

  fn validate_i16_type(&self, value: &str) -> Option<XrfError> {
    match value.parse::<i16>() {
      Ok(_) => None,
      Err(_) => Some(self.validation_error(&format!(
        "Invalid value, signed 16 bit number is expected, got '{value}'"
      ))),
    }
  }

  fn validate_u8_type(&self, value: &str) -> Option<XrfError> {
    match value.parse::<u8>() {
      Ok(_) => None,
      Err(_) => Some(self.validation_error(&format!(
        "Invalid value, unsigned 8 bit number is expected, got '{value}'"
      ))),
    }
  }

  fn validate_i8_type(&self, value: &str) -> Option<XrfError> {
    match value.parse::<i8>() {
      Ok(_) => None,
      Err(_) => Some(self.validation_error(&format!(
        "Invalid value, signed 8 bit number is expected, got '{value}'"
      ))),
    }
  }

  fn validate_bool_type(&self, value: &str) -> Option<XrfError> {
    match value.parse::<bool>() {
      Ok(_) => None,
      Err(_) => Some(self.validation_error(&format!("Invalid value, boolean is expected, got '{value}'"))),
    }
  }

  /// Validate if provided value is correct comma separated vector.
  /// Expected value like `x,y,z` in f32 format.
  fn validate_vector_type(&self, value: &str) -> Option<XrfError> {
    self.validate_fixed_float_list_type(value, 3)
  }

  /// Validate if provided value is correct comma separated rgb.
  /// Expected value like `r,g,b` in f32 format.
  fn validate_rgb_type(&self, value: &str) -> Option<XrfError> {
    self.validate_fixed_float_list_type(value, 3)
  }

  /// Validate if provided value is correct comma separated rgba.
  /// Expected value like `r,g,b,a` in f32 format.
  fn validate_rgba_type(&self, value: &str) -> Option<XrfError> {
    self.validate_fixed_float_list_type(value, 4)
  }

  /// Validate if provided value is correct list of floats with defined len.
  fn validate_fixed_float_list_type(&self, value: &str, len: usize) -> Option<XrfError> {
    let parsed_values: Vec<f32> = value.split(',').filter_map(|x| x.trim().parse::<f32>().ok()).collect();

    if parsed_values.len() != len {
      Some(self.validation_error(&format!(
        "Invalid value, comma separated {len} float values expected, got '{value}'"
      )))
    } else {
      None
    }
  }

  /// Validate if provided value is correct enumeration defined field.
  fn validate_enum_type(&self, value: &str) -> Option<XrfError> {
    match &self.data_type {
      LtxFieldDataType::TypeEnum(allowed_values) => {
        if allowed_values.is_empty() {
          Some(self.validation_error("Unexpected enum check - list of possible values is empty"))
        } else if allowed_values.contains(&value.into()) {
          None
        } else {
          Some(self.validation_error(&format!(
            "Invalid value, one of possible values [{}] expected, got '{value}'",
            allowed_values.join(",")
          )))
        }
      }
      _ => Some(self.validation_error("Unexpected enum type check, trying to validate enum with non-enum field")),
    }
  }

  /// Validate if provided value matches tuple description.
  fn validate_tuple_type(&self, value: &str) -> Option<XrfError> {
    match &self.data_type {
      LtxFieldDataType::TypeTuple(types, types_raw, separator) => {
        if types.is_empty() {
          Some(self.validation_error("Unexpected tuple check - list of possible values is empty"))
        } else {
          let values: Vec<&str> = value.split(separator.as_char()).map(|it| it.trim()).collect();
          let values_count: usize = values.len();
          let required_values_count: usize = types_raw
            .iter()
            .filter(|it| !LtxFieldDataType::is_field_data_optional(it))
            .count();

          if values_count < required_values_count || values_count > types.len() {
            Some(self.validation_error(&format!(
              "Invalid value, {} {} separated values required, provided {} ('{}' in '{}' field)",
              required_values_count,
              separator.as_name(),
              values_count,
              value,
              types_raw.join(", ")
            )))
          } else {
            let loop_length: usize = cmp::max(values.len(), types.len());

            // Validate all provided values.
            for it in 0..loop_length {
              let current_type: &LtxFieldDataType = types.get(it).unwrap();
              let current_value: &str = values.get(it).copied().unwrap_or("");

              // Skip optional values with empty data, do not validate.
              if current_value.is_empty() && LtxFieldDataType::is_field_data_optional(types_raw.get(it).unwrap()) {
                continue;
              }

              if let Some(error) = self.validate_data_entry_by_type(current_type, current_value) {
                return Some(self.validation_error(&format!(
                  "Tuple error [{}] at [{}] - {}",
                  types_raw.join(","),
                  it,
                  error
                )));
              }
            }

            None
          }
        }
      }
      _ => Some(self.validation_error("Unexpected tuple type check, trying to validate enum with non-enum field")),
    }
  }

  /// Validate if provided value matches const description.
  fn validate_const(&self, value: &str) -> Option<XrfError> {
    match &self.data_type {
      LtxFieldDataType::TypeConst(const_value) => {
        if const_value.is_empty() {
          Some(self.validation_error("Unexpected const check - value is empty"))
        } else if const_value != value {
          Some(self.validation_error(&format!(
            "Invalid value - constant '{const_value} is expected, got '{value}'"
          )))
        } else {
          None
        }
      }
      _ => Some(self.validation_error("Unexpected tuple type check, trying to validate enum with non-enum field")),
    }
  }

  fn validate_section_type(&self, _: &str) -> Option<XrfError> {
    None
  }

  fn validate_section_type_defined(&self, ltx: &Ltx, value: &str) -> Option<XrfError> {
    if ltx.has_section(value) {
      None
    } else {
      Some(self.validation_error(&format!(
        "Unexpected value - section [{value}] is not defined in file scope"
      )))
    }
  }

  fn validate_condlist_type(&self, value: &str) -> Option<XrfError> {
    if value.is_empty() && !self.is_optional {
      return Some(self.validation_error("Invalid value - condlist is expected, got empty field"));
    }

    if value.is_empty() {
      return None;
    }

    Condlist::parse(value)
      .err()
      .map(|error| self.validation_error(&error.to_string()))
  }

  fn validate_string_type(&self, value: &str) -> Option<XrfError> {
    if value.is_empty() && !self.is_optional {
      Some(self.validation_error("Invalid value - string is expected, got empty field"))
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::LtxFieldScheme;
  use crate::Ltx;
  use crate::scheme::LtxFieldDataType;
  use crate::scheme::TupleSeparator;

  #[test]
  fn test_u32_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeI32);

    assert!(scheme.validate_u32_type(&u32::MIN.to_string()).is_none());
    assert!(scheme.validate_u32_type(&i32::MAX.to_string()).is_none());

    assert!(scheme.validate_u32_type(&((u32::MIN as i64) - 1).to_string()).is_some());
    assert!(scheme.validate_u32_type(&((u32::MAX as i64) + 1).to_string()).is_some());

    assert!(scheme.validate_u32_type("a").is_some());
    assert!(scheme.validate_u32_type("true").is_some());
    assert!(scheme.validate_u32_type(",").is_some());
    assert!(scheme.validate_u32_type("").is_some());
  }

  #[test]
  fn test_i32_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeI32);

    assert!(scheme.validate_i32_type(&i32::MIN.to_string()).is_none());
    assert!(scheme.validate_i32_type(&i32::MAX.to_string()).is_none());

    assert!(scheme.validate_i32_type(&((i32::MIN as i64) - 1).to_string()).is_some());
    assert!(scheme.validate_i32_type(&((i32::MAX as i64) + 1).to_string()).is_some());

    assert!(scheme.validate_i32_type("a").is_some());
    assert!(scheme.validate_i32_type("true").is_some());
    assert!(scheme.validate_i32_type(",").is_some());
    assert!(scheme.validate_i32_type("").is_some());
  }

  #[test]
  fn test_u16_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeU16);

    assert!(scheme.validate_u16_type(&u16::MIN.to_string()).is_none());
    assert!(scheme.validate_u16_type(&u16::MAX.to_string()).is_none());

    assert!(scheme.validate_u16_type(&((u16::MIN as i32) - 1).to_string()).is_some());
    assert!(scheme.validate_u16_type(&((u16::MAX as i32) + 1).to_string()).is_some());

    assert!(scheme.validate_u16_type("a").is_some());
    assert!(scheme.validate_u16_type("true").is_some());
    assert!(scheme.validate_u16_type(",").is_some());
    assert!(scheme.validate_u16_type("").is_some());
  }

  #[test]
  fn test_i16_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeI16);

    assert!(scheme.validate_i16_type(&i16::MIN.to_string()).is_none());
    assert!(scheme.validate_i16_type(&i16::MAX.to_string()).is_none());

    assert!(scheme.validate_i16_type(&((i16::MIN as i32) - 1).to_string()).is_some());
    assert!(scheme.validate_i16_type(&((i16::MAX as i32) + 1).to_string()).is_some());

    assert!(scheme.validate_i16_type("a").is_some());
    assert!(scheme.validate_i16_type("true").is_some());
    assert!(scheme.validate_i16_type(",").is_some());
    assert!(scheme.validate_i16_type("").is_some());
  }

  #[test]
  fn test_u8_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeU8);

    assert!(scheme.validate_u8_type(&u8::MIN.to_string()).is_none());
    assert!(scheme.validate_u8_type(&u8::MAX.to_string()).is_none());

    assert!(scheme.validate_u8_type(&((u8::MIN as i32) - 1).to_string()).is_some());
    assert!(scheme.validate_u8_type(&((u8::MAX as i32) + 1).to_string()).is_some());

    assert!(scheme.validate_u8_type("a").is_some());
    assert!(scheme.validate_u8_type("true").is_some());
    assert!(scheme.validate_u8_type(",").is_some());
    assert!(scheme.validate_u8_type("").is_some());
  }

  #[test]
  fn test_i8_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeI8);

    assert!(scheme.validate_i8_type(&i8::MIN.to_string()).is_none());
    assert!(scheme.validate_i8_type(&i8::MAX.to_string()).is_none());

    assert!(scheme.validate_i8_type(&((i8::MIN as i32) - 1).to_string()).is_some());
    assert!(scheme.validate_i8_type(&((i8::MAX as i32) + 1).to_string()).is_some());

    assert!(scheme.validate_i8_type("a").is_some());
    assert!(scheme.validate_i8_type("true").is_some());
    assert!(scheme.validate_i8_type(",").is_some());
    assert!(scheme.validate_i8_type("").is_some());
  }

  #[test]
  fn test_bool_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeBool);

    assert!(scheme.validate_bool_type("true").is_none());
    assert!(scheme.validate_bool_type("false").is_none());

    assert!(scheme.validate_bool_type("1,2,3,4").is_some());
    assert!(scheme.validate_bool_type("1").is_some());
    assert!(scheme.validate_bool_type("0").is_some());
    assert!(scheme.validate_bool_type("").is_some());
  }

  #[test]
  fn test_condlist_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_type("test_section", "test_field", LtxFieldDataType::TypeCondlist);

    assert!(
      scheme
        .validate_condlist_type("{+info -other} enabled, fallback %=set_active_task(test)%")
        .is_none()
    );
    assert!(
      scheme
        .validate_condlist_type("{+info} %=mech_discount(0.95)%")
        .is_none()
    );

    assert!(scheme.validate_condlist_type("").is_some());
    assert!(scheme.validate_condlist_type("{+} enabled").is_some());
    assert!(scheme.validate_condlist_type("enabled %effect").is_some());
  }
  #[test]
  fn test_string_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_type("test_section", "test_field", LtxFieldDataType::TypeString);

    assert!(scheme.validate_string_type("true").is_none());
    assert!(scheme.validate_string_type("false").is_none());
    assert!(scheme.validate_string_type("0").is_none());
    assert!(scheme.validate_string_type("1").is_none());
    assert!(scheme.validate_string_type("-1").is_none());
    assert!(scheme.validate_string_type(",").is_none());
    assert!(scheme.validate_string_type(",,,,,,").is_none());

    assert!(scheme.validate_string_type("").is_some());
  }

  #[test]
  fn test_string_array_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_array_type("test_section", "test_field", LtxFieldDataType::TypeString);

    let ltx: Ltx = Ltx::new();

    assert!(scheme.validate_value(&ltx, "true").is_none());
    assert!(scheme.validate_value(&ltx, "false").is_none());
    assert!(scheme.validate_value(&ltx, "0").is_none());
    assert!(scheme.validate_value(&ltx, "1").is_none());
    assert!(scheme.validate_value(&ltx, "-1").is_none());

    assert!(scheme.validate_value(&ltx, ",").is_some());
    assert!(scheme.validate_value(&ltx, ",,,,,,,,,").is_some());
    assert!(scheme.validate_value(&ltx, ",,,   , ,     ,").is_some());
    assert!(scheme.validate_value(&ltx, "").is_some());
  }

  #[test]
  fn test_string_optional_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeString);

    assert!(scheme.validate_string_type("true").is_none());
    assert!(scheme.validate_string_type("false").is_none());
    assert!(scheme.validate_string_type("0").is_none());
    assert!(scheme.validate_string_type("1").is_none());
    assert!(scheme.validate_string_type("-1").is_none());
    assert!(scheme.validate_string_type(",").is_none());
    assert!(scheme.validate_string_type(",,,,,,").is_none());

    assert!(scheme.validate_string_type("").is_none());
  }

  #[test]
  fn test_vector_validation() {
    let scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeVector);

    assert!(scheme.validate_vector_type("1,2,3").is_none());
    assert!(scheme.validate_vector_type("-2,2.5,-0.0025").is_none());
    assert!(scheme.validate_vector_type("-1.5,-2.5,-3.1").is_none());

    assert!(scheme.validate_vector_type("").is_some());
    assert!(scheme.validate_vector_type("1,2,3,4").is_some());
    assert!(scheme.validate_vector_type("1,2").is_some());
    assert!(scheme.validate_vector_type("-1,2.0").is_some());
    assert!(scheme.validate_vector_type("a,b,c").is_some());
  }

  #[test]
  fn test_enum_validation() {
    let mut scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeVector);

    assert!(scheme.validate_enum_type("a").is_some());

    scheme.data_type = LtxFieldDataType::TypeEnum(vec![String::from("a"), String::from("b_c"), String::from("d")]);

    assert!(scheme.validate_enum_type("a").is_none());
    assert!(scheme.validate_enum_type("b_c").is_none());
    assert!(scheme.validate_enum_type("d").is_none());

    assert!(scheme.validate_enum_type("").is_some());
    assert!(scheme.validate_enum_type("e").is_some());
    assert!(scheme.validate_enum_type("f").is_some());
    assert!(scheme.validate_enum_type("1").is_some());
  }

  #[test]
  fn test_tuple_validation() {
    let mut scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeVector);

    assert!(scheme.validate_enum_type("a").is_some());

    scheme.data_type = LtxFieldDataType::TypeTuple(
      vec![
        LtxFieldDataType::TypeF32,
        LtxFieldDataType::TypeString,
        LtxFieldDataType::TypeBool,
      ],
      vec![String::from("f32"), String::from("string"), String::from("bool")],
      TupleSeparator::Comma,
    );

    assert!(scheme.validate_tuple_type("15.25, abc, true").is_none());
    assert!(scheme.validate_tuple_type("-12.50, def, false").is_none());
    assert!(scheme.validate_tuple_type("0, a, true").is_none());

    assert!(scheme.validate_tuple_type("").is_some());
    assert!(scheme.validate_tuple_type("e").is_some());
    assert!(scheme.validate_tuple_type("f").is_some());
    assert!(scheme.validate_tuple_type("1").is_some());
    assert!(scheme.validate_tuple_type("10,,true_true").is_some());
    assert!(scheme.validate_tuple_type("10,,1").is_some());
    assert!(scheme.validate_tuple_type("a,b,c").is_some());
    assert!(scheme.validate_tuple_type("a,b,c,d").is_some());
  }

  #[test]
  fn test_tuple_validation_optionals() {
    let mut scheme: LtxFieldScheme =
      LtxFieldScheme::new_with_optional_type("test_section", "test_field", LtxFieldDataType::TypeVector);

    assert!(scheme.validate_enum_type("a").is_some());

    scheme.data_type = LtxFieldDataType::TypeTuple(
      vec![
        LtxFieldDataType::TypeF32,
        LtxFieldDataType::TypeString,
        LtxFieldDataType::TypeString,
        LtxFieldDataType::TypeBool,
      ],
      vec![
        String::from("f32"),
        String::from("string"),
        String::from("?string"),
        String::from("?bool"),
      ],
      TupleSeparator::Comma,
    );

    assert!(scheme.validate_tuple_type("15.25, xxx, abc, true").is_none());
    assert!(scheme.validate_tuple_type("-12.50, xxx, def, false").is_none());
    assert!(scheme.validate_tuple_type("0, xxx, a, true").is_none());
    assert!(scheme.validate_tuple_type("0, xxx, a").is_none());
    assert!(scheme.validate_tuple_type("0, xxx").is_none());
    assert!(scheme.validate_tuple_type("0, true,,").is_none());
    assert!(scheme.validate_tuple_type("0, 1,,").is_none());

    assert!(scheme.validate_tuple_type(", xxx, a, true").is_some());
    assert!(scheme.validate_tuple_type(", a, true").is_some());
    assert!(scheme.validate_tuple_type(", , , true").is_some());
    assert!(scheme.validate_tuple_type(", , ,").is_some());
    assert!(scheme.validate_tuple_type("").is_some());
    assert!(scheme.validate_tuple_type("e").is_some());
    assert!(scheme.validate_tuple_type("f").is_some());
    assert!(scheme.validate_tuple_type("10,xxx,,,,,,,,,,").is_some());
    assert!(scheme.validate_tuple_type("10,xxx,test,true,true").is_some());
    assert!(scheme.validate_tuple_type("10,xxx,test,true_true").is_some());
    assert!(scheme.validate_tuple_type("10xxx,,,true_true").is_some());
    assert!(scheme.validate_tuple_type("10,,,1").is_some());
    assert!(scheme.validate_tuple_type("a,b").is_some());
    assert!(scheme.validate_tuple_type("a,b,c").is_some());
    assert!(scheme.validate_tuple_type("a,b,c,d").is_some());
  }

  #[test]
  fn test_pipe_tuple_condlist_validation() {
    let data_type: LtxFieldDataType =
      LtxFieldDataType::from_field_data("on_signal", "test_section", "tuple@pipe:string,condlist")
        .expect("Expected pipe tuple type to parse");
    let scheme: LtxFieldScheme = LtxFieldScheme::new_with_optional_type("test_section", "on_signal", data_type);

    assert!(
      scheme
        .validate_data_entry_by_type(&scheme.data_type, "signal | {+info} next_state")
        .is_none()
    );
    assert!(
      scheme
        .validate_data_entry_by_type(&scheme.data_type, " | {+info} next_state")
        .is_none()
    );
    assert!(
      scheme
        .validate_data_entry_by_type(&scheme.data_type, "signal | {+}")
        .is_some()
    );
    assert!(
      scheme
        .validate_data_entry_by_type(&scheme.data_type, "signal |")
        .is_none()
    );
  }
}
