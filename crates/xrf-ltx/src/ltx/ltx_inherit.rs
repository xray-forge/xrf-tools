use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path_or;

use crate::document::LtxCheck;
use crate::ltx::{Ltx, LtxSections, Section};
use crate::syntax::VIRTUAL_LTX_PATH;

/// Converter object to process and inject all inherit section statements.
#[derive(Default)]
pub struct LtxInheritConvertor {}

impl LtxInheritConvertor {
  fn new() -> Self {
    Self {}
  }

  /// Cast LTX file to fully parsed with include sections.
  pub fn convert(ltx: Ltx) -> XrfResult<Ltx> {
    Self::new().convert_ltx(ltx)
  }
}

impl LtxInheritConvertor {
  /// Convert ltx file with inclusion of inherited sections.
  fn convert_ltx(&self, mut ltx: Ltx) -> XrfResult<Ltx> {
    if !ltx.includes.is_empty() {
      return Err(XrfError::new_convert_error(
        "Failed to equipment ltx file, not processed include statements detected on inheritance conversion",
      ));
    }

    if ltx.is_check_skipped(LtxCheck::Inheritance) {
      return Ok(ltx);
    }

    // Nothing to parse - no child sections.
    if ltx.sections.is_empty() {
      return Ok(ltx);
    }

    let mut new_sections: LtxSections = Default::default();

    self.inherit_sections(&ltx, &mut new_sections)?;

    ltx.sections = new_sections;

    Ok(ltx)
  }

  fn inherit_sections(&self, ltx: &Ltx, destination: &mut LtxSections) -> XrfResult {
    for (section_name, _) in &ltx.sections {
      Self::inherit_section(ltx, destination, section_name)?;
    }

    Ok(())
  }

  fn inherit_section(ltx: &Ltx, destination: &mut LtxSections, section_name: &str) -> XrfResult {
    let section: &Section = match ltx.sections.get(section_name) {
      None => {
        return Err(XrfError::new_convert_error(format!(
          "Failed to inherit unknown section [{section_name}] when reading ltx file ({})",
          format_path_or(ltx.path.as_deref(), VIRTUAL_LTX_PATH)
        )));
      }
      Some(it) => it,
    };

    // No need in recursive check multiple times with re-declaration.
    if destination.contains_key(section_name) {
      return Ok(());
    }

    if section.inherited.is_empty() {
      destination.insert(section_name.into(), section.clone());
    } else {
      for inherited in &section.inherited {
        if section_name == inherited {
          return Err(XrfError::new_convert_error(format!(
            "Failed to inherit section '{inherited}' in '{section_name}', cannot inherit self"
          )));
        }

        Self::inherit_section(ltx, destination, inherited)?;
      }

      let mut new_props: Section = Default::default();

      for inherited in &section.inherited {
        for (key, value) in destination.get(inherited).unwrap() {
          new_props.insert(key, value)
        }
      }

      for (key, value) in section {
        new_props.insert(key, value)
      }

      new_props.inherited = Default::default();

      destination.insert(section_name.into(), new_props);
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use xrf_error::XrfResult;

  use crate::Section;
  use crate::ltx::Ltx;

  #[test]
  fn test_inheritance_chain() {
    let input = "
[base_1]
a = 1
b = 2

[base_2]:base_1
b = 3
c = 4

[base_3]:base_2
c = 10
d = 20

[target]:base_3
e = 100
";

    let ltx: XrfResult<Ltx> = Ltx::read_from_str(input);

    assert!(ltx.is_ok());

    let ltx: XrfResult<Ltx> = ltx.unwrap().into_inherited();

    assert!(ltx.is_ok());

    let output: Ltx = ltx.unwrap();
    assert_eq!(output.len(), 4);

    let target: &Section = output.section("target").unwrap();

    assert_eq!(target.len(), 5);
    assert_eq!(target.get("a"), Some("1"));
    assert_eq!(target.get("b"), Some("3"));
    assert_eq!(target.get("c"), Some("10"));
    assert_eq!(target.get("d"), Some("20"));
    assert_eq!(target.get("e"), Some("100"));

    let base_3: &Section = output.section("base_3").unwrap();

    assert_eq!(base_3.len(), 4);
    assert_eq!(base_3.get("a"), Some("1"));
    assert_eq!(base_3.get("b"), Some("3"));
    assert_eq!(base_3.get("c"), Some("10"));
    assert_eq!(base_3.get("d"), Some("20"));

    let base_2: &Section = output.section("base_2").unwrap();

    assert_eq!(base_2.len(), 3);
    assert_eq!(base_2.get("a"), Some("1"));
    assert_eq!(base_2.get("b"), Some("3"));
    assert_eq!(base_2.get("c"), Some("4"));

    let base_2: &Section = output.section("base_1").unwrap();

    assert_eq!(base_2.len(), 2);
    assert_eq!(base_2.get("a"), Some("1"));
    assert_eq!(base_2.get("b"), Some("2"));
  }
}

#[cfg(test)]
mod reported_path_test {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_error::XrfResult;
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use crate::ltx::Ltx;

  /// Reads a file that inherits an undeclared section, and returns the rendered diagnostic.
  fn inherit_error_for(root: &Path, name: impl AsRef<Path>) -> XrfResult<String> {
    fs::create_dir_all(root)?;

    let path: PathBuf = root.join(name);

    fs::write(&path, "[child]:missing\n")?;

    let error: String = Ltx::read_from_file_standard(&path)
      .expect_err("Expected inheriting an undeclared section to fail")
      .to_string();

    fs::remove_dir_all(root)?;

    Ok(error)
  }

  #[test]
  fn names_the_file_the_undeclared_section_was_inherited_in() -> XrfResult {
    let root: PathBuf = build_absolute_generated_test_resource_path("inherit/named");
    let error: String = inherit_error_for(&root, "broken.ltx")?;

    assert!(error.contains("Failed to inherit unknown section [missing]"), "{error}");
    assert!(error.contains("broken.ltx"), "{error}");

    Ok(())
  }

  /// Linux filenames are bytes, not text, so a valid path can still not be valid Unicode. Rendering the
  /// inheritance diagnostic used to unwrap `Path::to_str` on it and abort the process instead of
  /// returning the error the caller asked for.
  #[test]
  #[cfg(target_os = "linux")]
  fn returns_an_inheritance_error_for_a_file_whose_name_is_not_valid_unicode() -> XrfResult {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let root: PathBuf = build_absolute_generated_test_resource_path("inherit/non_utf8");
    let name: &OsStr = OsStr::from_bytes(b"\xffbroken.ltx");

    assert!(name.to_str().is_none());

    let error: String = inherit_error_for(&root, name)?;

    assert!(error.contains("Failed to inherit unknown section [missing]"), "{error}");
    assert!(error.contains('\u{fffd}'), "{error}");

    Ok(())
  }
}
