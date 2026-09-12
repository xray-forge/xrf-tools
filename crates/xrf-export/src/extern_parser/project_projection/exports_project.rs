use std::fs;
use std::path::PathBuf;

use xrf_error::{XrfError, XrfResult};

use super::{ExportDescriptor, ExportSourceDescriptor};
use crate::extern_manifest::ExternManifest;

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

/// The source text that declares one extern.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSourceContent {
  pub name: String,
  pub path: String,
  pub line: usize,
  pub end_line: usize,
  pub content: String,
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

#[cfg(test)]
mod tests {
  use std::fs;

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::*;
  use crate::extern_parser::project_projection::{ExportContractDescriptor, ExportReturnDescriptor};

  fn descriptor(name: &str, path: &str, line: usize, end_line: usize) -> ExportDescriptor {
    ExportDescriptor {
      name: name.into(),
      description: None,
      source: ExportSourceDescriptor {
        path: path.into(),
        line,
        column: 1,
        end_line,
      },
      contract: ExportContractDescriptor::Callable {
        parameters: Vec::new(),
        returns: ExportReturnDescriptor {
          typing: "void".into(),
          description: None,
        },
      },
    }
  }

  fn project_with(root: PathBuf, declarations: Vec<ExportDescriptor>) -> ExportsProject {
    ExportsProject {
      root,
      declarations,
      manifest: ExternManifest::default(),
    }
  }

  fn write_sample(name: &str, contents: &str) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("project-source/{name}"));

    fs::create_dir_all(&root).expect("temporary directory");
    fs::write(root.join("declaration.ts"), contents).expect("temporary declaration");

    root
  }

  #[test]
  fn reads_every_line_of_a_multiline_declaration() {
    let root: PathBuf = write_sample(
      "multiline",
      "const a = 1;\nextern(\"on_game\", () => {\n  log(\"a\");\n  log(\"b\");\n});\nconst b = 2;\n",
    );
    let project: ExportsProject = project_with(root, vec![descriptor("on_game", "declaration.ts", 2, 5)]);

    let source: ExportSourceContent = project.read_declaration_source("on_game").unwrap();

    assert_eq!(
      source.content,
      "extern(\"on_game\", () => {\n  log(\"a\");\n  log(\"b\");\n});"
    );
    assert_eq!(source.line, 2);
    assert_eq!(source.end_line, 5);
  }

  #[test]
  fn reads_a_single_line_declaration() {
    let root: PathBuf = write_sample("single", "const a = 1;\nextern(\"value\", 5 as number);\n");
    let project: ExportsProject = project_with(root, vec![descriptor("value", "declaration.ts", 2, 2)]);

    assert_eq!(
      project.read_declaration_source("value").unwrap().content,
      "extern(\"value\", 5 as number);"
    );
  }

  #[test]
  fn refuses_a_name_the_project_does_not_declare() {
    // The name is the only key into the file system here, so an unknown one must not read anything.
    let root: PathBuf = write_sample("unknown", "extern(\"known\", () => {});\n");
    let project: ExportsProject = project_with(root, vec![descriptor("known", "declaration.ts", 1, 1)]);

    let error = project.read_declaration_source("../../secrets").unwrap_err();

    assert!(error.to_string().contains("is not part of this project"), "{error}");
  }

  #[test]
  fn returns_what_remains_when_the_file_shrank_since_parsing() {
    // Lines were recorded at parse time and the file may have been edited since.
    let root: PathBuf = write_sample("shrunk", "extern(\"trimmed\", () => {});\n");
    let project: ExportsProject = project_with(root, vec![descriptor("trimmed", "declaration.ts", 1, 40)]);

    assert_eq!(
      project.read_declaration_source("trimmed").unwrap().content,
      "extern(\"trimmed\", () => {});"
    );
  }

  #[test]
  fn reports_a_declaration_whose_file_is_gone() {
    let root: PathBuf = write_sample("missing", "extern(\"gone\", () => {});\n");
    let project: ExportsProject = project_with(root, vec![descriptor("gone", "removed.ts", 1, 1)]);

    assert!(project.read_declaration_source("gone").is_err());
  }
}
