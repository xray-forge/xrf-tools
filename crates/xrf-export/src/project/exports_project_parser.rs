use std::path::Path;

use xrf_error::XrfResult;

use crate::parse::{ExternManifestParser, ParsedExternManifest};
use crate::project::descriptor::ExportDescriptor;
use crate::project::exports_project::ExportsProject;

/// Projects canonical externs into the application-facing exports project.
#[derive(Default)]
pub struct ExportsProjectParser;

impl ExportsProjectParser {
  /// Create a parser that builds the application-facing exports project.
  pub fn new() -> Self {
    Self
  }

  /// Scan one project and project every extern into its application-facing form, sorted by name.
  pub fn parse_project_from_path<P: AsRef<Path>>(&self, path: P) -> XrfResult<ExportsProject> {
    let root: &Path = path.as_ref();
    let ParsedExternManifest { manifest, parsed } = ExternManifestParser::new().parse_directory(root)?;
    let mut declarations: Vec<ExportDescriptor> = parsed.into_iter().map(ExportDescriptor::from).collect();

    declarations.sort_by(|left, right| left.name.cmp(&right.name));

    Ok(ExportsProject {
      root: root.to_path_buf(),
      declarations,
      manifest,
    })
  }
}
