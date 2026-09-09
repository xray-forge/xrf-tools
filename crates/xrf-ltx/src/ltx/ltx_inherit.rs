use std::collections::HashMap;
use std::sync::Arc;

use fxhash::FxBuildHasher;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path_or;

use crate::dialect::{LtxFieldOrigin, LtxProvenance};
use crate::document::LtxCheck;
use crate::ltx::{Ltx, LtxSections, Section};
use crate::syntax::VIRTUAL_LTX_PATH;

/// Which section writes each field of one resolved section, keyed by the field's own shared handle.
type SectionAttribution = HashMap<Arc<str>, Arc<str>, FxBuildHasher>;

/// Converter object to process and inject all inherit section statements.
#[derive(Default)]
pub struct LtxInheritConvertor {
  /// Which section writes each field, per resolved section, or nothing when no caller asked.
  ///
  /// Built alongside the fields rather than reconstructed afterwards, because after inheritance a copied field is
  /// indistinguishable from a written one - that is the whole point of the copy.
  attributions: Option<HashMap<String, SectionAttribution, FxBuildHasher>>,
}

impl LtxInheritConvertor {
  fn new() -> Self {
    Default::default()
  }

  /// The same, recording which section writes each resolved field.
  fn new_recording() -> Self {
    Self {
      attributions: Some(Default::default()),
    }
  }

  /// Cast LTX file to fully parsed with include sections.
  pub fn convert(ltx: Ltx) -> XrfResult<Ltx> {
    Ok(Self::new().convert_ltx(ltx)?.0)
  }

  /// The same, answering also with where each resolved field is written.
  ///
  /// Costs one map entry per resolved field, which is why it is a separate door rather than a return value everyone
  /// pays for. See [`crate::LtxResolveRequest`].
  pub(crate) fn convert_recording(ltx: Ltx) -> XrfResult<(Ltx, LtxProvenance)> {
    let (ltx, provenance) = Self::new_recording().convert_ltx(ltx)?;

    Ok((ltx, provenance.unwrap_or_default()))
  }
}

impl LtxInheritConvertor {
  /// Convert ltx file with inclusion of inherited sections.
  fn convert_ltx(mut self, mut ltx: Ltx) -> XrfResult<(Ltx, Option<LtxProvenance>)> {
    if !ltx.includes.is_empty() {
      return Err(XrfError::new_convert_error(
        "Failed to equipment ltx file, not processed include statements detected on inheritance conversion",
      ));
    }

    // The last step of standard resolution either way, so this is where the growth room every section was sized for
    // stops being needed. Here rather than in the dialect, because the free readers finish here too.
    if ltx.is_check_skipped(LtxCheck::Inheritance) {
      ltx.shrink_to_fit();

      // Nothing was copied, so every field is written where it sits - which is an answer, not an absence.
      let provenance: Option<LtxProvenance> = self.attributions.is_some().then(|| Self::all_declared(&ltx));

      return Ok((ltx, provenance));
    }

    // Nothing to parse - no child sections.
    if ltx.sections.is_empty() {
      let provenance: Option<LtxProvenance> = self.attributions.is_some().then(LtxProvenance::default);

      return Ok((ltx, provenance));
    }

    let mut new_sections: LtxSections = Default::default();

    self.inherit_sections(&ltx, &mut new_sections)?;

    ltx.sections = new_sections;
    ltx.shrink_to_fit();

    // Folded after the sections are in place, because an inherited field's file is the declaring file of the section
    // that writes it, and only the resolved map knows that for every ancestor.
    let provenance: Option<LtxProvenance> = self.attributions.take().map(|attributions| {
      let mut provenance: LtxProvenance = Self::fold_attributions(&ltx, attributions);

      provenance.shrink_to_fit();

      provenance
    });

    Ok((ltx, provenance))
  }

  /// Every field written where it sits, which is what a tree with no inheritance applied comes to.
  fn all_declared(ltx: &Ltx) -> LtxProvenance {
    let mut provenance: LtxProvenance = LtxProvenance::default();

    for (section_name, section) in &ltx.sections {
      let name: Arc<str> = Arc::from(section_name.as_str());
      let mut fields: HashMap<Arc<str>, LtxFieldOrigin, FxBuildHasher> = Default::default();

      for (key, _) in section.iter_shared() {
        fields.insert(
          Arc::clone(key),
          LtxFieldOrigin::Declared {
            file: section.origin.clone(),
          },
        );
      }

      provenance.insert_section(name, fields);
    }

    provenance
  }

  /// Turns "which section writes this field" into "and which file that section was declared in".
  fn fold_attributions(ltx: &Ltx, attributions: HashMap<String, SectionAttribution, FxBuildHasher>) -> LtxProvenance {
    let mut provenance: LtxProvenance = LtxProvenance::default();

    for (section_name, attributed) in attributions {
      let name: Arc<str> = Arc::from(section_name.as_str());
      let mut fields: HashMap<Arc<str>, LtxFieldOrigin, FxBuildHasher> = Default::default();

      for (key, writer) in attributed {
        let origin: LtxFieldOrigin = if *writer == *name {
          LtxFieldOrigin::Declared {
            file: ltx.sections.get(section_name.as_str()).and_then(|it| it.origin.clone()),
          }
        } else {
          LtxFieldOrigin::Inherited {
            file: ltx.sections.get(&*writer).and_then(|it| it.origin.clone()),
            section: writer,
          }
        };

        fields.insert(key, origin);
      }

      provenance.insert_section(name, fields);
    }

    provenance
  }

  fn inherit_sections(&mut self, ltx: &Ltx, destination: &mut LtxSections) -> XrfResult {
    for (section_name, _) in &ltx.sections {
      self.inherit_section(ltx, destination, section_name)?;
    }

    Ok(())
  }

  fn inherit_section(&mut self, ltx: &Ltx, destination: &mut LtxSections, section_name: &str) -> XrfResult {
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
      self.attribute_own(section_name, section, None);

      destination.insert(section_name.into(), section.clone());
    } else {
      for inherited in &section.inherited {
        if section_name == inherited {
          return Err(XrfError::new_convert_error(format!(
            "Failed to inherit section '{inherited}' in '{section_name}', cannot inherit self"
          )));
        }

        self.inherit_section(ltx, destination, inherited)?;
      }

      let mut new_props: Section = Default::default();

      // Shared, not copied: a parent's fields end up in every child that inherits it, and on a weapon tree that is the
      // same text stored dozens of times over.
      for inherited in &section.inherited {
        new_props.extend_shared(destination.get(inherited).unwrap());
      }

      new_props.extend_shared(section);
      new_props.inherited = Default::default();
      // The child's own declaring file, not a parent's: inheritance copies fields in, it does not move the header.
      new_props.origin = section.origin.clone();

      // Parents folded left then the section's own fields on top, which is the order the two `extend_shared` passes
      // above apply, so the recorded winner is the one the value came from.
      self.attribute_own(section_name, section, Some(&section.inherited));

      destination.insert(section_name.into(), new_props);
    }

    Ok(())
  }

  /// Records which section writes each field this one ends up with.
  ///
  /// A parent contributes what its own resolution attributed, not its name, so a field two levels up reports where it
  /// is written rather than the step it arrived through.
  fn attribute_own(&mut self, section_name: &str, section: &Section, parents: Option<&[String]>) {
    let Some(attributions) = self.attributions.as_mut() else {
      return;
    };

    let name: Arc<str> = Arc::from(section_name);
    let mut attributed: SectionAttribution = Default::default();

    if let Some(parents) = parents {
      for parent in parents {
        if let Some(inherited) = attributions.get(parent.as_str()) {
          for (key, writer) in inherited {
            attributed.insert(Arc::clone(key), Arc::clone(writer));
          }
        }
      }
    }

    for (key, _) in section.iter_shared() {
      attributed.insert(Arc::clone(key), Arc::clone(&name));
    }

    attributions.insert(String::from(section_name), attributed);
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
