use xrf_ltx::{
  LTX_SCHEME_FIELD, LTX_SYMBOL_ANY, LtxFieldScheme, LtxResolution, LtxSectionScheme, LtxSectionSchemes, Section,
};

use crate::resolved::{LtxResolvedField, LtxResolvedFieldOrigin};
use crate::scheme::ltx_scheme_report::{LtxSchemeFieldDeclaration, LtxSchemeFieldReport, LtxSectionSchemeReport};

/// Reads one resolved section against the scheme that judges it.
///
/// Reached through [`crate::LtxRootReader`], which owns the root the section belongs to; see its `read_section_scheme`.
pub(crate) struct LtxSchemeReader {}

impl LtxSchemeReader {
  /// What one section is judged by, and how it measures against that.
  pub(crate) fn read_section_scheme(
    entry: &str,
    resolution: &LtxResolution,
    declarations: Option<&LtxSectionSchemes>,
    name: &str,
  ) -> Option<LtxSectionSchemeReport> {
    let section: &Section = resolution.ltx.section(name)?;
    let scheme: Option<&str> = section.get(LTX_SCHEME_FIELD);
    let declaration: Option<&LtxSectionScheme> = scheme
      .zip(declarations)
      .and_then(|(scheme, declarations)| declarations.get(scheme));

    Some(LtxSectionSchemeReport {
      entry: String::from(entry),
      fields: Self::read_fields(resolution, name, section, declaration),
      inherited_from: Self::read_binding_owner(resolution, name),
      is_declared: declaration.is_some(),
      is_strict: declaration.is_some_and(|declaration| declaration.is_strict),
      scheme: scheme.map(String::from),
      section: String::from(name),
    })
  }

  /// Every field the scheme declares and every field the section holds, merged into one list of rows.
  fn read_fields(
    resolution: &LtxResolution,
    name: &str,
    section: &Section,
    declaration: Option<&LtxSectionScheme>,
  ) -> Vec<LtxSchemeFieldReport> {
    let mut fields: Vec<LtxSchemeFieldReport> = Vec::new();

    if let Some(declaration) = declaration {
      for (field, declared) in &declaration.fields {
        // The catch-all is not a field anybody wrote. It is reported on each row it types instead, where a reader
        // looking for the field by name in the scheme file will otherwise find nothing.
        if field == LTX_SYMBOL_ANY {
          continue;
        }

        fields.push(LtxSchemeFieldReport {
          declared: Some(Self::to_declaration(declared, false)),
          resolved: Self::to_resolved(resolution, name, section, field),
          name: String::from(field),
        });
      }
    }

    let any: Option<&LtxFieldScheme> = declaration.and_then(|declaration| declaration.fields.get(LTX_SYMBOL_ANY));

    for (field, _) in section {
      if declaration.is_some_and(|declaration| declaration.fields.contains_key(field)) {
        continue;
      }

      fields.push(LtxSchemeFieldReport {
        declared: any.map(|declared| Self::to_declaration(declared, true)),
        resolved: Self::to_resolved(resolution, name, section, field),
        name: String::from(field),
      });
    }

    fields
  }

  /// The section a binding is written in, when the section being read is not the one that writes it.
  ///
  /// Read from provenance rather than guessed from the parents: what a header names is where inheritance started, and
  /// the resolution records where the value it ended up with was actually written.
  fn read_binding_owner(resolution: &LtxResolution, name: &str) -> Option<String> {
    match resolution.get_origin(name, LTX_SCHEME_FIELD).into() {
      LtxResolvedFieldOrigin::Inherited { section, .. } => Some(section),
      _ => None,
    }
  }

  /// One declared field, as a record something outside Rust can render.
  fn to_declaration(declared: &LtxFieldScheme, is_any: bool) -> LtxSchemeFieldDeclaration {
    LtxSchemeFieldDeclaration {
      data_type: declared.data_type.to_string(),
      is_any,
      is_array: declared.is_array,
      is_optional: declared.is_optional,
    }
  }

  /// What the section holds for one field, with where that value was written.
  fn to_resolved(resolution: &LtxResolution, name: &str, section: &Section, field: &str) -> Option<LtxResolvedField> {
    section.get(field).map(|value| LtxResolvedField {
      key: String::from(field),
      origin: resolution.get_origin(name, field).into(),
      value: String::from(value),
    })
  }
}
