use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_ltx::{LtxDocumentSource, LtxResolution, Section};

use crate::resolved::{
  LtxResolvedDiagnostic, LtxResolvedField, LtxResolvedIndex, LtxResolvedIndexEntry, LtxResolvedSection,
};
use crate::structure::list_section_parents;

/// Where a declaring config's headers are read once and reused, keyed by that config's engine identity.
type DeclaredParents = HashMap<String, HashMap<String, Vec<String>>>;

/// Reads one resolution as an index and, on demand, as pages of section bodies.
///
/// Holds the source as well as the resolution because parents do not survive resolving - flattening inheritance is
/// what resolving is - so the only place a section's declared parents still exist is the header in its declaring
/// config. Those documents are already parsed and retained by whatever produced the resolution, so reading them back
/// costs a lookup rather than a read.
pub struct LtxResolvedReader<'a> {
  entry: &'a str,
  dialect: &'a str,
  resolution: &'a LtxResolution,
  source: &'a dyn LtxDocumentSource,
}

impl<'a> LtxResolvedReader<'a> {
  /// A reader over one resolved root.
  ///
  /// `dialect` is how the dialect that produced `resolution` names itself; the resolution does not carry it, and the
  /// caller that chose the dialect is the one place that knows.
  pub fn new(
    entry: &'a str,
    dialect: &'a str,
    resolution: &'a LtxResolution,
    source: &'a dyn LtxDocumentSource,
  ) -> Self {
    Self {
      dialect,
      entry,
      resolution,
      source,
    }
  }

  /// Every section of the root, named and counted.
  ///
  /// # Errors
  ///
  /// Returns an error when a declaring config cannot be read back.
  pub fn read_index(&self) -> XrfResult<LtxResolvedIndex> {
    let mut declared: DeclaredParents = HashMap::new();
    let mut sections: Vec<LtxResolvedIndexEntry> = Vec::with_capacity(self.resolution.ltx.len());

    for (name, section) in self.resolution.ltx.iter() {
      sections.push(LtxResolvedIndexEntry {
        field_count: section.len(),
        origin: section.get_origin().map(String::from),
        parents: self.read_parents(name, section, &mut declared)?,
        name: String::from(name),
      });
    }

    Ok(LtxResolvedIndex {
      dialect: String::from(self.dialect),
      diagnostics: self
        .resolution
        .diagnostics
        .iter()
        .map(LtxResolvedDiagnostic::from)
        .collect(),
      entry: String::from(self.entry),
      sections,
    })
  }

  /// The bodies of the named sections, in the order asked for.
  ///
  /// Addressed by name rather than by offset, so a filter applied on one side never has to be mirrored on the other
  /// and a page is always whole sections. A name the root does not hold is skipped rather than refused: a page request
  /// races an index the caller may have fetched before a reopen.
  ///
  /// # Errors
  ///
  /// Returns an error when a declaring config cannot be read back.
  pub fn read_sections(&self, names: &[&str]) -> XrfResult<Vec<LtxResolvedSection>> {
    let mut declared: DeclaredParents = HashMap::new();
    let mut sections: Vec<LtxResolvedSection> = Vec::with_capacity(names.len());

    for name in names {
      let Some(section) = self.resolution.ltx.section(name) else {
        continue;
      };

      sections.push(LtxResolvedSection {
        entry: String::from(self.entry),
        fields: self.read_fields(name, section),
        origin: section.get_origin().map(String::from),
        parents: self.read_parents(name, section, &mut declared)?,
        name: String::from(*name),
      });
    }

    Ok(sections)
  }

  /// One section's fields, each with whatever the resolution recorded about where it came from.
  fn read_fields(&self, name: &str, section: &Section) -> Vec<LtxResolvedField> {
    section
      .iter()
      .map(|(key, value)| LtxResolvedField {
        origin: self.resolution.get_origin(name, key).into(),
        key: String::from(key),
        value: String::from(value),
      })
      .collect()
  }

  /// The parents one section's header declared, reading its declaring config at most once per config.
  fn read_parents(&self, name: &str, section: &Section, declared: &mut DeclaredParents) -> XrfResult<Vec<String>> {
    let Some(origin) = section.get_origin() else {
      return Ok(Vec::new());
    };

    if !declared.contains_key(origin) {
      let headers: HashMap<String, Vec<String>> = match self.source.read_document(origin)? {
        Some(document) => list_section_parents(&document),
        None => HashMap::new(),
      };

      declared.insert(String::from(origin), headers);
    }

    Ok(
      declared
        .get(origin)
        .and_then(|headers| headers.get(name))
        .cloned()
        .unwrap_or_default(),
    )
  }
}
