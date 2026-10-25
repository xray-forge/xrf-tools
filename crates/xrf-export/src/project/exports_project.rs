use std::fs;
use std::path::PathBuf;

use xrf_error::{XrfError, XrfResult};

use crate::manifest::ExternManifest;
use crate::project::descriptor::{ExportDescriptor, ExportSourceDescriptor};
use crate::project::export_source_content::ExportSourceContent;

/// Parsed externs and the project they came from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportsProject {
  pub root: PathBuf,
  pub declarations: Vec<ExportDescriptor>,
  /// The canonical manifest these declarations are a projection of, kept so a surface can publish the artifact
  /// `xrf-cli externs export` writes without parsing the tree a second time and answering for a different read of it.
  #[serde(skip)]
  #[cfg_attr(feature = "typescript-bindings", specta(skip))]
  pub manifest: ExternManifest,
}

impl ExportsProject {
  /// Read back the source that declares one extern.
  ///
  /// Resolved from the name rather than from a path supplied by the caller: this Interface may only read
  /// what this project already parsed, so the surface cannot be used to read arbitrary files.
  pub fn read_declaration_source(&self, name: &str) -> XrfResult<ExportSourceContent> {
    let declaration: &ExportDescriptor = self
      .declarations
      .iter()
      .find(|declaration: &&ExportDescriptor| declaration.name == name)
      .ok_or_else(|| XrfError::new_not_found_error(format!("Export '{}' is not part of this project.", name)))?;

    let source: &ExportSourceDescriptor = &declaration.source;
    // Stored with `/` separators regardless of platform, which `join` handles on both.
    let path: PathBuf = self.root.join(&source.path);
    let content: String = fs::read_to_string(&path)?;

    Ok(ExportSourceContent {
      name: declaration.name.clone(),
      path: source.path.clone(),
      line: source.line,
      end_line: source.end_line,
      content: Self::take_lines(&content, source.line, source.end_line),
    })
  }

  /// Take an inclusive, one-based line range, tolerating a range the file no longer covers.
  ///
  /// The lines were recorded when the project was parsed, and the file may have been edited since.
  /// Returning what is still there beats failing outright, since the view is informational.
  fn take_lines(content: &str, line: usize, end_line: usize) -> String {
    let start: usize = line.saturating_sub(1);

    content
      .lines()
      .skip(start)
      .take(end_line.saturating_sub(start))
      .collect::<Vec<&str>>()
      .join("\n")
  }
}
