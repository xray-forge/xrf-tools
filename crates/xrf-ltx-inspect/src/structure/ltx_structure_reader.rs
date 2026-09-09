use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LTX_SCHEME_FIELD, Ltx, LtxDocument, LtxDocumentSource, LtxItemKind, LtxResolution, Section};

use crate::structure::{
  LtxFileStructure, LtxStructureInclude, LtxStructureParent, LtxStructureParseError, LtxStructureScheme,
  LtxStructureSection,
};

/// Reads one config as written, judged against the resolution its entry point produced.
///
/// Takes the resolution rather than resolving anything: whether a parent resolves and which scheme a section ends up
/// bound to are both questions about the whole root, and a file read on its own can answer neither. One reader serves
/// every file of one root, which is also what keeps the answers consistent across them.
pub struct LtxStructureReader<'a> {
  resolution: &'a LtxResolution,
  source: &'a dyn LtxDocumentSource,
  declared_schemes: &'a [&'a str],
}

impl<'a> LtxStructureReader<'a> {
  /// A reader over one resolved root and the world its configs came from.
  pub fn new(resolution: &'a LtxResolution, source: &'a dyn LtxDocumentSource) -> Self {
    Self {
      declared_schemes: &[],
      resolution,
      source,
    }
  }

  /// The scheme names the project declares, so a binding can be reported as bound or dangling.
  ///
  /// Left empty by default rather than defaulted to "declared": a reader given nothing says every binding is
  /// undeclared, which is what a project with no scheme files actually means.
  pub fn with_declared_schemes(mut self, declared_schemes: &'a [&'a str]) -> Self {
    self.declared_schemes = declared_schemes;

    self
  }

  /// One config's structure, or the parse error that stopped it from having one.
  ///
  /// A file that will not parse is answered, not refused: the viewer still opens it, shows its text and marks the line.
  /// Anything else that goes wrong - a config out of scope, an unreadable include - is a failure of the read rather
  /// than a finding about the file, and comes back as an error.
  ///
  /// # Errors
  ///
  /// Returns an error when the config is not in scope, or when resolving one of its includes fails.
  pub fn read(&self, path: &str, entry_points: &[String]) -> XrfResult<LtxFileStructure> {
    let document: Arc<LtxDocument> = match self.source.read_document(path) {
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
      includes: self.read_includes(path, &document)?,
      parse_error: None,
      path: String::from(path),
      sections: self.read_sections(&document),
    })
  }

  /// Every header of one document, with the parents and binding the resolution gives it.
  fn read_sections(&self, document: &LtxDocument) -> Vec<LtxStructureSection> {
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
            resolves: self.resolution.ltx.section(parent).is_some(),
          })
          .collect(),
        scheme: self.read_scheme(name),
      });
    }

    sections
  }

  /// The scheme one section is bound to, read off the resolution so an inherited binding counts.
  fn read_scheme(&self, section: &str) -> Option<LtxStructureScheme> {
    let resolved: &Section = self.resolution.ltx.section(section)?;
    let name: &str = resolved.get(LTX_SCHEME_FIELD)?;

    Some(LtxStructureScheme {
      is_declared: self.declared_schemes.contains(&name),
      name: String::from(name),
    })
  }

  /// Every `#include` of one document, with the configs it reached.
  ///
  /// A statement expands to itself whether or not the file exists, so the expansion is filtered by what the world
  /// actually holds - which is what makes an empty list mean "this include reached nothing".
  fn read_includes(&self, path: &str, document: &LtxDocument) -> XrfResult<Vec<LtxStructureInclude>> {
    let directory: &str = Ltx::directory_of(path);
    let mut includes: Vec<LtxStructureInclude> = Vec::new();

    for item in document.get_items() {
      let LtxItemKind::Include { path: statement, .. } = &item.kind else {
        continue;
      };

      let mut resolved: Vec<String> = Vec::new();

      for candidate in self.source.resolve_include(directory, statement)? {
        match self.source.read_document(&candidate) {
          Ok(Some(_)) => resolved.push(candidate),
          Ok(None) => {}
          // The file is there. That it will not parse is a finding about that file, not about this statement, and
          // reporting the include as unresolved would send a reader to the wrong line.
          Err(XrfError::LtxParse { .. }) => resolved.push(candidate),
          Err(error) => return Err(error),
        }
      }

      includes.push(LtxStructureInclude {
        line: item.span.line,
        resolved,
        statement: String::from(&**statement),
      });
    }

    Ok(includes)
  }
}
