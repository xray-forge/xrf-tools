use indexmap::map::Entry;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path_or;
use xrf_vfs::{XrayLogicalPath, XrayLookupScope, XrayVfs};

use crate::file::file_configuration::constants::{
  LTX_SCHEME_FIELD, LTX_SCHEME_STRICT_FIELD, LTX_SYMBOL_SCHEME, VIRTUAL_LTX_PATH,
};
use crate::file::file_section::section::Section;
use crate::file::ltx::Ltx;
use crate::file::types::LtxSectionSchemes;
use crate::scheme::field_data_type::LtxFieldDataType;
use crate::scheme::field_scheme::LtxFieldScheme;
use crate::scheme::section_scheme::LtxSectionScheme;

/// Parser of LTX scheme definitions.
#[derive(Clone, Debug)]
pub struct LtxSchemeParser {}

impl LtxSchemeParser {
  /// Parses LTX section schemes from logical paths in a mounted VFS.
  ///
  /// # Errors
  ///
  /// Returns an error when a scheme cannot be read or contains invalid, duplicate, or malformed declarations.
  pub fn parse_from_vfs(
    vfs: &XrayVfs,
    scope: &XrayLookupScope,
    files: &[XrayLogicalPath],
  ) -> XrfResult<LtxSectionSchemes> {
    let mut schemes: LtxSectionSchemes = Default::default();

    for file in files {
      let ltx: Ltx = Ltx::read_from_vfs_standard(vfs, scope, file.as_str())?;

      for (name, section) in &ltx {
        if !name.starts_with(LTX_SYMBOL_SCHEME) {
          return Err(XrfError::new_convert_error(format!(
            "Failed to parse ltx schemes - scheme section declaration should be prefixed with $, \
             got [{name}]"
          )));
        }

        match schemes.entry(name.into()) {
          Entry::Occupied(_) => {
            return Err(XrfError::new_convert_error(format!(
              "Failed to parse ltx schemes - duplicate declaration of [{name}] section when reading '{}'",
              format_path_or(ltx.path.as_deref(), VIRTUAL_LTX_PATH)
            )));
          }
          Entry::Vacant(entry) => {
            entry.insert(Self::parse_section_scheme(name, section)?);
          }
        }
      }
    }

    Ok(schemes)
  }

  /// Parse scheme from section.
  fn parse_section_scheme(section_name: &str, section: &Section) -> XrfResult<LtxSectionScheme> {
    let mut scheme: LtxSectionScheme = LtxSectionScheme::new(section_name);

    // Insert default definition of $scheme field.
    scheme.fields.insert(
      LTX_SCHEME_FIELD.into(),
      LtxFieldScheme {
        data_type: LtxFieldDataType::TypeString,
        is_array: false,
        is_optional: false,
        name: LTX_SCHEME_FIELD.into(),
        section: section_name.into(),
      },
    );

    for (field_name, value) in section {
      match field_name {
        LTX_SCHEME_STRICT_FIELD => {
          scheme.is_strict = Self::parse_strict_mode(field_name, section_name, value)?;
        }
        _ => {
          scheme.fields.insert(
            field_name.into(),
            Self::parse_field_scheme(field_name, section_name, value)?,
          );
        }
      }
    }

    Ok(scheme)
  }

  /// Parse LTX field definition from section by field name.
  fn parse_field_scheme(field_name: &str, section_name: &str, field_data: &str) -> XrfResult<LtxFieldScheme> {
    let data_type: LtxFieldDataType = LtxFieldDataType::from_field_data(field_name, section_name, field_data)?;

    // Do not allow unknown typing.
    if data_type == LtxFieldDataType::TypeUnknown {
      return Err(XrfError::new_read_error(format!(
        "Invalid ltx [{section_name}] {field_name} configuration, unknown type '{field_data}' supplied",
      )));
    }

    Ok(LtxFieldScheme {
      data_type,
      is_array: LtxFieldDataType::is_field_data_array(field_data),
      is_optional: LtxFieldDataType::is_field_data_optional(field_data),
      name: field_name.into(),
      section: section_name.into(),
    })
  }

  /// Parse whether strict mode is activated for ltx scheme.
  fn parse_strict_mode(field_name: &str, section_name: &str, field_data: &str) -> XrfResult<bool> {
    field_data.parse::<bool>().map_err(|error| {
      XrfError::new_read_error(format!(
        "Invalid scheme declaration, unexpected value for [{section_name}] {field_name} - '{field_data}', boolean expected ({error})"
      ))
    })
  }
}

#[cfg(test)]
mod tests {
  use super::LtxSchemeParser;
  use crate::Section;

  #[test]
  fn parses_dollar_strict_scheme_metadata() {
    let mut section: Section = Section::new();

    section.insert("$strict", "true");

    let scheme = LtxSchemeParser::parse_section_scheme("$test", &section).unwrap();

    assert!(scheme.is_strict);
    assert!(!scheme.fields.contains_key("$strict"));
  }
}
