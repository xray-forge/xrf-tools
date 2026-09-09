use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{
  LTX_SCHEME_FIELD, LTX_SYMBOL_INCLUDE_WILDCARD, Ltx, LtxDocument, LtxDocumentSource, LtxItemKind, LtxResolution,
  Section,
};

use crate::structure::{
  LtxFileStructure, LtxStructureInclude, LtxStructureParent, LtxStructureParseError, LtxStructureScheme,
  LtxStructureSection,
};

/// Reads one config as written, judged against the resolution its entry point produced.
///
/// Reached through [`crate::LtxRootReader`], which owns the root this is judged against; see its `read_structure`.
pub(crate) struct LtxStructureReader {}

impl LtxStructureReader {
  /// One config as written, judged against the resolution its entry point produced.
  ///
  /// Reached through [`crate::LtxRootReader`], which owns the root this is judged against; see its `read_structure`.
  pub(crate) fn read_structure(
    resolution: &LtxResolution,
    source: &dyn LtxDocumentSource,
    declared_schemes: &[&str],
    path: &str,
    entry_points: &[String],
  ) -> XrfResult<LtxFileStructure> {
    let document: Arc<LtxDocument> = match source.read_document(path) {
      Ok(Some(document)) => document,
      Ok(None) => {
        return Err(XrfError::new_convert_error(format!(
          "Failed to read ltx structure, '{path}' is not in scope"
        )));
      }
      Err(XrfError::LtxParse { line, col, message }) => {
        return Ok(LtxFileStructure {
          entry_points: entry_points.to_vec(),
          includes: Vec::new(),
          parse_error: Some(LtxStructureParseError {
            column: col as u32,
            line: line as u32,
            message,
          }),
          path: String::from(path),
          sections: Vec::new(),
        });
      }
      Err(error) => return Err(error),
    };

    Ok(LtxFileStructure {
      entry_points: entry_points.to_vec(),
      includes: Self::read_includes(source, path, &document)?,
      parse_error: None,
      path: String::from(path),
      sections: Self::read_sections(resolution, declared_schemes, &document),
    })
  }

  /// Every header of one document, with the parents and binding the resolution gives it.
  fn read_sections(
    resolution: &LtxResolution,
    declared_schemes: &[&str],
    document: &LtxDocument,
  ) -> Vec<LtxStructureSection> {
    let mut sections: Vec<LtxStructureSection> = Vec::new();

    for item in document.get_items() {
      let LtxItemKind::Section {
        name,
        operation,
        parents,
        ..
      } = &item.kind
      else {
        continue;
      };

      sections.push(LtxStructureSection {
        line: item.span.line,
        name: String::from(&**name),
        operation: String::from(operation.as_prefix()),
        parents: parents
          .iter()
          .map(|parent| LtxStructureParent {
            name: String::from(&**parent),
            resolves: resolution.ltx.section(parent).is_some(),
          })
          .collect(),
        scheme: Self::read_scheme(resolution, declared_schemes, name),
      });
    }

    sections
  }

  /// The scheme one section is bound to, read off the resolution so an inherited binding counts.
  fn read_scheme(resolution: &LtxResolution, declared_schemes: &[&str], section: &str) -> Option<LtxStructureScheme> {
    let resolved: &Section = resolution.ltx.section(section)?;
    let name: &str = resolved.get(LTX_SCHEME_FIELD)?;

    Some(LtxStructureScheme {
      is_declared: declared_schemes.contains(&name),
      name: String::from(name),
    })
  }

  /// Every `#include` of one document, with the configs it reached.
  ///
  /// An empty list means the statement reached nothing, which is a defect a reader can act on.
  fn read_includes(
    source: &dyn LtxDocumentSource,
    path: &str,
    document: &LtxDocument,
  ) -> XrfResult<Vec<LtxStructureInclude>> {
    let directory: &str = Ltx::directory_of(path);
    let mut includes: Vec<LtxStructureInclude> = Vec::new();

    for item in document.get_items() {
      let LtxItemKind::Include { path: statement, .. } = &item.kind else {
        continue;
      };

      includes.push(LtxStructureInclude {
        line: item.span.line,
        resolved: Self::resolve_reached(source, directory, statement)?,
        statement: String::from(&**statement),
      });
    }

    Ok(includes)
  }

  /// The configs one `#include` statement actually reached.
  ///
  /// A wildcard expands to what the world holds, so its expansion is the answer. A named statement expands to itself
  /// whether or not the file exists (`LtxDocumentSource::resolve_include`), so that one - and only that one - is probed.
  /// Probing a wildcard too would read every config a `w_*.ltx` matches to learn what listing the directory
  /// already said - hundreds of them on a real weapons tree.
  fn resolve_reached(source: &dyn LtxDocumentSource, directory: &str, statement: &str) -> XrfResult<Vec<String>> {
    let candidates: Vec<String> = source.resolve_include(directory, statement)?;

    if statement.contains(LTX_SYMBOL_INCLUDE_WILDCARD) {
      return Ok(candidates);
    }

    let mut reached: Vec<String> = Vec::new();

    for candidate in candidates {
      match source.read_document(&candidate) {
        Ok(Some(_)) => reached.push(candidate),
        Ok(None) => {}
        // The file is there. That it will not parse is a finding about that file, not about this statement, and
        // reporting the include as unresolved would send a reader to the wrong line.
        Err(XrfError::LtxParse { .. }) => reached.push(candidate),
        Err(error) => return Err(error),
      }
    }

    Ok(reached)
  }
}
