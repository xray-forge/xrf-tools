mod callable_parser;
mod declaration_parser;
mod diagnostics;
mod jsdoc_parser;
mod project_projection;
mod type_renderer;
mod value_parser;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use project_projection::{
  ExportContractDescriptor, ExportDescriptor, ExportParameterDescriptor, ExportReturnDescriptor, ExportSourceContent,
  ExportSourceDescriptor, ExportsProject, ExportsProjectParser,
};
use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_typescript::{TypeScriptSymbolResolver, parse_typescript_file};
use xrf_utils::{format_path, to_portable_path_string};

use crate::extern_manifest::{ExternExport, ExternManifest, ParsedExternManifest};

/// Parses TypeScript extern declarations into the canonical manifest model.
///
/// Source paths in the manifest are always relative to the declarations root.
#[derive(Default)]
pub struct ExternManifestParser;

impl ExternManifestParser {
  /// Create an extern manifest parser.
  pub fn new() -> Self {
    Self
  }

  /// Scan `declarations_root` and parse every eligible TypeScript declaration.
  pub fn parse_directory(&self, declarations_root: &Path) -> XrfResult<ParsedExternManifest> {
    // todo: Allow single file parsing?
    if !declarations_root.is_dir() {
      return Err(XrfError::new_invalid_error(format!(
        "Extern source root '{}' is not a directory.",
        format_path(declarations_root),
      )));
    }

    let files: Vec<PathBuf> = self.read_source_files(declarations_root);

    self.parse_files(&files, declarations_root)
  }

  fn parse_files(&self, files: &[PathBuf], declarations_root: &Path) -> XrfResult<ParsedExternManifest> {
    let mut parsed = Vec::new();

    for path in files {
      let source = parse_typescript_file(path)?;
      let source_path: String = self.normalize_declaration_path(path, declarations_root)?;
      let symbol_resolver: TypeScriptSymbolResolver =
        TypeScriptSymbolResolver::discover(path.parent().expect("TypeScript source path has a parent directory"))?;
      let mut declarations = declaration_parser::ExternDeclarationParser::new(
        &source.source_map,
        &source.comments,
        path,
        &source_path,
        &symbol_resolver,
      )
      .parse(&source.program)?;

      parsed.append(&mut declarations);
    }

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

    Ok(ParsedExternManifest {
      manifest: ExternManifest { exports },
      parsed,
    })
  }

  /// Return whether a TypeScript source can contribute an extern declaration.
  pub fn is_source_path(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "ts")
      && !path.file_name().is_some_and(|name| {
        name.to_string_lossy().ends_with(".test.ts") || name.to_string_lossy().ends_with(".spec.ts")
      })
      && !path.components().any(|component| component.as_os_str() == "__test__")
  }

  fn read_source_files(&self, declarations_root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = WalkDir::new(declarations_root)
      .into_iter()
      .filter_entry(|entry: &DirEntry| Self::should_visit(entry, declarations_root))
      .filter_map(Result::ok)
      .map(|entry| entry.into_path())
      .filter(|path| Self::is_source_path(path))
      .filter(|path| Self::contains_extern_call(path))
      .collect();

    files.sort();

    files
  }

  fn contains_extern_call(path: &Path) -> bool {
    fs::read_to_string(path).is_ok_and(|source: String| {
      source
        .match_indices("extern")
        .any(|(offset, _)| source[offset + "extern".len()..].trim_start().starts_with('('))
    })
  }

  fn should_visit(entry: &DirEntry, declarations_root: &Path) -> bool {
    entry.path() == declarations_root
      || !entry.file_type().is_dir()
      || !matches!(
        entry.file_name().to_str(),
        Some(".git" | "node_modules" | "target" | "dist")
      )
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

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::ExternManifestParser;
  use crate::ExternExport;

  fn create_test_root(name: &str) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("extern-parser/{name}"));
    fs::create_dir_all(&root).unwrap();
    root
  }

  fn write_source(root: &Path, name: &str, source: &str) {
    let path: PathBuf = root.join(name);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
  }

  #[test]
  fn parses_direct_object_and_asserted_externs_with_docs() {
    let root: PathBuf = create_test_root("manifest");

    write_source(
      &root,
      "externs.ts",
      r#"
        export {};
        /**
         * Shared callbacks.
         * @param id - Callback identifier.
         * @returns Callback result.
         */
        extern("callbacks", {
          run: (id: TId): boolean => true,
        });

        /** Numeric data. */
        extern("data.value", rawValue as { readonly id: string });
        extern("data.checkers", rawValue as Record<EAchievement, () => boolean>);
      "#,
    );

    let parsed = ExternManifestParser::new().parse_directory(&root).unwrap();
    let callback = parsed.manifest.exports.get("callbacks.run").unwrap();
    let ExternExport::Callable(callback) = callback else {
      panic!("Expected callable extern");
    };

    assert_eq!(callback.params[0].name, "id");
    assert_eq!(callback.params[0].type_name, "TId");
    assert_eq!(callback.source, "externs.ts");
    assert_eq!(callback.params[0].doc.as_deref(), Some("Callback identifier."));
    assert_eq!(callback.returns, "boolean");
    assert_eq!(
      callback
        .doc
        .as_ref()
        .and_then(|documentation| documentation.returns.as_deref()),
      Some("Callback result.")
    );
    assert!(matches!(
      parsed.manifest.exports.get("data.value"),
      Some(ExternExport::Value(_))
    ));
    let ExternExport::Value(value) = parsed.manifest.exports.get("data.value").unwrap() else {
      panic!("Expected value extern");
    };
    assert_eq!(value.type_name, "{ readonly id: string }");

    let ExternExport::Value(checkers) = parsed.manifest.exports.get("data.checkers").unwrap() else {
      panic!("Expected value extern");
    };

    assert_eq!(checkers.type_name, "Record<EAchievement, () => boolean>");
  }

  #[test]
  fn parses_externs_from_a_script_without_module_syntax() {
    let root: PathBuf = create_test_root("script-extern");

    write_source(
      &root,
      "effects.ts",
      "extern(\"xr_effects.give_item\", (section: string): void => {});",
    );

    let parsed = ExternManifestParser::new().parse_directory(&root).unwrap();
    let ExternExport::Callable(callable) = parsed.manifest.exports.get("xr_effects.give_item").unwrap() else {
      panic!("Expected callable extern");
    };

    assert_eq!(callable.params[0].name, "section");
    assert_eq!(callable.params[0].type_name, "string");
    assert_eq!(callable.returns, "void");
  }

  #[test]
  fn uses_unknown_for_missing_callable_types_and_rejects_duplicate_names() {
    let root: PathBuf = create_test_root("invalid");
    write_source(&root, "missing.ts", "export {}; extern(\"test\", (value) => true);");

    let parser = ExternManifestParser::new();
    let parsed = parser.parse_directory(&root).unwrap();

    let ExternExport::Callable(callable) = parsed.manifest.exports.get("test").unwrap() else {
      panic!("Expected callable extern");
    };

    assert_eq!(callable.params[0].type_name, "unknown");
    assert_eq!(callable.returns, "unknown");

    write_source(
      &root,
      "missing.ts",
      "export {}; extern(\"test\", (): boolean => true); extern(\"test\", (): boolean => false);",
    );

    assert!(
      parser
        .parse_directory(&root)
        .unwrap_err()
        .to_string()
        .contains("Duplicate extern")
    );
  }

  #[test]
  fn explains_an_unresolved_function_reference() {
    let root: PathBuf = create_test_root("function-reference");
    write_source(&root, "callbacks.ts", "export {}; extern(\"callbacks.run\", run);");

    let error = ExternManifestParser::new()
      .parse_directory(&root)
      .unwrap_err()
      .to_string();

    assert!(
      error.contains("function reference `run` for extern 'callbacks.run' needs a type"),
      "{error}"
    );
    assert!(error.contains("`run as (arg: Type) => ReturnType`"), "{error}");
  }

  #[test]
  fn resolves_imported_functions_in_extern_objects() {
    let root: PathBuf = create_test_root("imported-function");
    write_source(
      &root,
      "src/tsconfig.json",
      r#"{
        "compilerOptions": {
          "baseUrl": "./engine",
          "paths": { "@/*": ["../*"] }
        }
      }"#,
    );
    write_source(&root, "src/engine/callbacks/index.ts", "export * from \"./handlers\";");
    write_source(
      &root,
      "src/engine/callbacks/handlers.ts",
      r#"
        export function run(object: GameObject, count?: number): Nillable<string> { return null; }

        export const conditions: Record<EAchievement, () => boolean> = {};

        export const config = {
          nested_conditions: {
            enabled: (): boolean => true,
          },
        };
      "#,
    );
    write_source(
      &root,
      "src/engine/declarations/callbacks.ts",
      r#"
        import { conditions, config, run } from "@/engine/callbacks";

        extern("callbacks", { run: run });
        extern("callbacks.conditions", conditions);
        extern("callbacks.nested_conditions", config.nested_conditions);
      "#,
    );

    let parsed = ExternManifestParser::new()
      .parse_directory(&root.join("src/engine/declarations"))
      .unwrap();
    let ExternExport::Callable(callback) = parsed.manifest.exports.get("callbacks.run").unwrap() else {
      panic!("Expected callable extern");
    };

    assert_eq!(callback.source, "callbacks.ts");
    assert_eq!(callback.params[0].name, "object");
    assert_eq!(callback.params[0].type_name, "GameObject");
    assert_eq!(callback.params[1].name, "count");
    assert_eq!(callback.params[1].optional, Some(true));
    assert_eq!(callback.params[1].type_name, "number");
    assert_eq!(callback.returns, "Nillable<string>");

    let ExternExport::Value(conditions) = parsed.manifest.exports.get("callbacks.conditions").unwrap() else {
      panic!("Expected value extern");
    };
    assert_eq!(conditions.type_name, "Record<EAchievement, () => boolean>");

    let ExternExport::Value(nested_conditions) = parsed.manifest.exports.get("callbacks.nested_conditions").unwrap()
    else {
      panic!("Expected value extern");
    };
    assert_eq!(nested_conditions.type_name, "{ enabled: () => boolean; }");
  }

  #[test]
  fn resolves_local_function_references() {
    let root: PathBuf = create_test_root("local-function");
    write_source(
      &root,
      "callbacks.ts",
      r#"
        export {};

        const run = (value: string): boolean => true;

        extern("callbacks.run", run);
      "#,
    );

    let parsed = ExternManifestParser::new().parse_directory(&root).unwrap();
    let ExternExport::Callable(callback) = parsed.manifest.exports.get("callbacks.run").unwrap() else {
      panic!("Expected callable extern");
    };

    assert_eq!(callback.params[0].name, "value");
    assert_eq!(callback.params[0].type_name, "string");
    assert_eq!(callback.returns, "boolean");
  }

  #[test]
  fn excludes_test_sources() {
    assert!(!ExternManifestParser::is_source_path(Path::new(
      "declarations/example.test.ts"
    )));
    assert!(!ExternManifestParser::is_source_path(Path::new(
      "declarations/example.spec.ts"
    )));
    assert!(!ExternManifestParser::is_source_path(Path::new(
      "declarations/__test__/example.ts"
    )));
    assert!(ExternManifestParser::is_source_path(Path::new(
      "declarations/example.ts"
    )));
  }

  #[test]
  fn rejects_a_missing_source_root() {
    let root: PathBuf = build_absolute_generated_test_resource_path("extern-parser/missing");

    let error = ExternManifestParser::new()
      .parse_directory(&root)
      .unwrap_err()
      .to_string();

    assert!(error.contains("is not a directory"), "{error}");
  }
}
