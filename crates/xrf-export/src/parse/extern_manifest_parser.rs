use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_typescript::{TypeScriptSource, TypeScriptSymbolResolver};
use xrf_utils::{format_path, to_portable_path_string};

use crate::manifest::{ExternExport, ExternManifest};
use crate::parse::declaration::{ExternDeclarationParser, ExternStatement};
use crate::parse::parsed_extern::ParsedExtern;
use crate::parse::parsed_extern_manifest::ParsedExternManifest;

/// Parses TypeScript extern declarations into the canonical manifest model.
///
/// Source paths in the manifest are always relative to the declarations root.
#[derive(Default)]
pub struct ExternManifestParser;

impl ExternManifestParser {
  /// Directories a declarations tree never holds sources in.
  const SKIPPED_DIRECTORIES: [&'static str; 4] = [".git", "node_modules", "target", "dist"];

  /// A directory whose sources are tests, whatever they declare.
  const TEST_DIRECTORY: &'static str = "__test__";

  /// Endings that mark a source as a test.
  const TEST_SUFFIXES: [&'static str; 2] = [".test.ts", ".spec.ts"];

  /// Create an extern manifest parser.
  pub fn new() -> Self {
    Self
  }

  /// Scan `declarations_root` and parse every eligible TypeScript declaration.
  pub fn parse_directory(&self, declarations_root: &Path) -> XrfResult<ParsedExternManifest> {
    if !declarations_root.is_dir() {
      return Err(XrfError::new_io_error(
        format!(
          "Extern source root '{}' is not a directory.",
          format_path(declarations_root),
        ),
        ErrorKind::NotADirectory,
      ));
    }

    let files: Vec<PathBuf> = self.read_source_files(declarations_root)?;

    self.parse_files(&files, declarations_root)
  }

  /// Return whether a TypeScript source can contribute an extern declaration.
  pub fn is_source_path(path: &Path) -> bool {
    XrayExtension::Ts.matches_path(path)
      && !path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();

        Self::TEST_SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
      })
      && !path
        .components()
        .any(|component| component.as_os_str() == Self::TEST_DIRECTORY)
  }

  fn parse_files(&self, files: &[PathBuf], declarations_root: &Path) -> XrfResult<ParsedExternManifest> {
    let mut statements: Vec<ExternStatement> = Vec::new();

    for path in files {
      let source: TypeScriptSource = TypeScriptSource::parse_file(path)?;
      let source_path: String = self.normalize_declaration_path(path, declarations_root)?;
      let symbol_resolver: TypeScriptSymbolResolver =
        TypeScriptSymbolResolver::discover(path.parent().expect("TypeScript source path has a parent directory"))?;

      statements.extend(
        ExternDeclarationParser::new(
          &source.source_map,
          &source.comments,
          path,
          &source_path,
          &symbol_resolver,
        )
        .parse(&source.program)?,
      );
    }

    let mut parsed: Vec<ParsedExtern> = statements
      .iter()
      .flat_map(|statement| statement.externs.iter().cloned())
      .collect();

    parsed.sort_by(|left, right| left.name.cmp(&right.name));

    let mut exports: BTreeMap<String, ExternExport> = BTreeMap::new();

    for declaration in &parsed {
      if let Some(existing) = exports.insert(declaration.name.clone(), declaration.export.clone()) {
        return Err(XrfError::new_invalid_error(format!(
          "Duplicate extern '{}' declared in '{}' and '{}'.",
          declaration.name,
          existing.source(),
          declaration.export.source(),
        )));
      }
    }

    Self::check_overlaps(&statements)?;

    Ok(ParsedExternManifest {
      manifest: ExternManifest { exports },
      parsed,
    })
  }

  /// Refuses two calls of which the game would lose or fail one (`ExternStatement::writes_over`): the manifest would
  /// list both, and at most one of them reaches the game.
  ///
  /// Statements arrive file by file in path order and in source order within a file, which is the order one module
  /// runs them in.
  fn check_overlaps(statements: &[ExternStatement]) -> XrfResult {
    for (index, earlier) in statements.iter().enumerate() {
      for later in &statements[index + 1..] {
        if !earlier.writes_over(later) {
          continue;
        }

        let (outer, inner) = if earlier.key.len() <= later.key.len() {
          (earlier, later)
        } else {
          (later, earlier)
        };

        return Err(XrfError::new_invalid_error(format!(
          "Extern '{}' at {} writes over extern '{}' at {}: each assigns its whole key, so the game keeps whichever \
           runs last, and a call into a value that is not a table fails.",
          inner.key, inner.location, outer.key, outer.location,
        )));
      }
    }

    Ok(())
  }

  /// The declaration sources under the root, in path order.
  fn read_source_files(&self, declarations_root: &Path) -> XrfResult<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(declarations_root)
      .into_iter()
      .filter_entry(|entry: &DirEntry| Self::should_visit(entry, declarations_root))
    {
      let path: PathBuf = entry
        .map_err(|error| {
          XrfError::new_io_error(
            format!("Failed to walk extern sources: {error}"),
            error.io_error().map_or(ErrorKind::Other, std::io::Error::kind),
          )
        })?
        .into_path();

      if Self::is_source_path(&path) && Self::contains_extern_call(&path)? {
        files.push(path);
      }
    }

    files.sort();

    Ok(files)
  }

  fn contains_extern_call(path: &Path) -> XrfResult<bool> {
    let source: String = fs::read_to_string(path).map_err(|error| {
      XrfError::new_io_error(
        format!("Failed to read extern source '{}': {error}", format_path(path)),
        error.kind(),
      )
    })?;

    Ok(
      source
        .match_indices(ExternDeclarationParser::CALLEE)
        .any(|(offset, callee)| source[offset + callee.len()..].trim_start().starts_with('(')),
    )
  }

  fn should_visit(entry: &DirEntry, declarations_root: &Path) -> bool {
    entry.path() == declarations_root
      || !entry.file_type().is_dir()
      || !entry
        .file_name()
        .to_str()
        .is_some_and(|name| Self::SKIPPED_DIRECTORIES.contains(&name))
  }

  fn normalize_declaration_path(&self, path: &Path, declarations_root: &Path) -> XrfResult<String> {
    let relative: &Path = path.strip_prefix(declarations_root).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "Declaration '{}' is outside declarations root '{}'.",
        format_path(path),
        format_path(declarations_root),
      ))
    })?;

    Ok(to_portable_path_string(relative))
  }
}
