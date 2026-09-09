use xrf_error::XrfResult;
use xrf_ltx::{LtxDocumentSource, LtxResolution, Section};

use crate::resolved::{
  LtxResolvedDiagnostic, LtxResolvedField, LtxResolvedIndex, LtxResolvedIndexEntry, LtxResolvedSection,
};
use crate::structure::LtxDeclaredParents;

/// Reads one resolution as an index, and as pages of section bodies.
///
/// Reached through [`crate::LtxRootReader`], which owns the root and the header cache.
pub(crate) struct LtxResolvedReader {}

impl LtxResolvedReader {
  /// One resolution as an index of every section it holds.
  ///
  /// Reached through [`crate::LtxRootReader`], which owns the root and the header cache; see its `read_index`.
  pub(crate) fn read_index(
    entry: &str,
    dialect: &str,
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    declared: &mut LtxDeclaredParents,
  ) -> XrfResult<LtxResolvedIndex> {
    let mut sections: Vec<LtxResolvedIndexEntry> = Vec::with_capacity(resolution.ltx.len());

    for (name, section) in resolution.ltx.iter() {
      sections.push(LtxResolvedIndexEntry {
        field_count: section.len(),
        origin: section.get_origin().map(String::from),
        parents: Self::read_parents(source, declared, name, section)?,
        name: String::from(name),
      });
    }

    Ok(LtxResolvedIndex {
      dialect: String::from(dialect),
      diagnostics: resolution.diagnostics.iter().map(LtxResolvedDiagnostic::from).collect(),
      entry: String::from(entry),
      sections,
    })
  }

  /// The bodies of the named sections, in the order asked for.
  ///
  /// Reached through [`crate::LtxRootReader`]; see its `read_sections`.
  pub(crate) fn read_sections(
    entry: &str,
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    declared: &mut LtxDeclaredParents,
    names: &[&str],
  ) -> XrfResult<Vec<LtxResolvedSection>> {
    let mut sections: Vec<LtxResolvedSection> = Vec::with_capacity(names.len());

    for name in names {
      let Some(section) = resolution.ltx.section(name) else {
        continue;
      };

      sections.push(LtxResolvedSection {
        entry: String::from(entry),
        fields: Self::read_fields(resolution, name, section),
        origin: section.get_origin().map(String::from),
        parents: Self::read_parents(source, declared, name, section)?,
        name: String::from(*name),
      });
    }

    Ok(sections)
  }

  /// One section's fields, each with whatever the resolution recorded about where it came from.
  fn read_fields(resolution: &LtxResolution, name: &str, section: &Section) -> Vec<LtxResolvedField> {
    section
      .iter()
      .map(|(key, value)| LtxResolvedField {
        origin: resolution.get_origin(name, key).into(),
        key: String::from(key),
        value: String::from(value),
      })
      .collect()
  }

  /// The parents one section's header declared, or nothing when no dialect stamped a declaring config on it.
  fn read_parents(
    source: &dyn LtxDocumentSource,
    declared: &mut LtxDeclaredParents,
    name: &str,
    section: &Section,
  ) -> XrfResult<Vec<String>> {
    match section.get_origin() {
      Some(origin) => declared.of(source, origin, name),
      None => Ok(Vec::new()),
    }
  }
}
