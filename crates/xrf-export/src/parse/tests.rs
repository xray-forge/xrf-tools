use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfError;
use xrf_test_utils::GeneratedResources;

use crate::{ExternExport, ExternManifestParser};

fn create_test_root(name: &str) -> PathBuf {
  GeneratedResources::create_dir(&format!("extern-parser/{name}")).unwrap()
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

  // A duplicate is content the parser judged, so it must not arrive as an `Io` failure.
  let error: XrfError = parser.parse_directory(&root).unwrap_err();

  assert!(error.to_string().contains("Duplicate extern"), "{error}");
  assert!(matches!(error, XrfError::Invalid { .. }), "{error}");
}

// `extern` assigns its whole key, so a call writing inside another's key, or a second object at the same key, leaves
// the game whichever module loaded last.
#[test]
fn rejects_calls_that_write_over_each_other() {
  let root: PathBuf = create_test_root("overlapping");

  write_source(&root, "pda.ts", "extern(\"pda\", { open: (): void => {} });");
  write_source(&root, "pda_extra.ts", "extern(\"pda.close\", (): void => {});");

  let error: XrfError = ExternManifestParser::new().parse_directory(&root).unwrap_err();

  assert!(matches!(error, XrfError::Invalid { .. }), "{error}");
  assert!(
    error
      .to_string()
      .contains("Extern 'pda.close' at pda_extra.ts:1:1 writes over extern 'pda' at pda.ts:1:1"),
    "{error}"
  );

  write_source(&root, "pda_extra.ts", "extern(\"pda\", { close: (): void => {} });");

  let error: XrfError = ExternManifestParser::new().parse_directory(&root).unwrap_err();

  assert!(error.to_string().contains("writes over extern 'pda'"), "{error}");

  // An empty object writes nothing the manifest lists, and still replaces the whole table.
  write_source(&root, "pda_extra.ts", "extern(\"pda\", {});");

  assert!(ExternManifestParser::new().parse_directory(&root).is_err());
}

// One module runs its calls in order: a later one may add to the table an earlier object made, but one written before
// that object, or into a value that is not a table, is lost or fails.
#[test]
fn orders_the_calls_of_one_module_as_it_runs_them() {
  let root: PathBuf = create_test_root("one-module");

  write_source(
    &root,
    "pda.ts",
    "extern(\"pda\", { open: (): void => {} }); extern(\"pda.close\", (): void => {});",
  );

  assert_eq!(
    ExternManifestParser::new()
      .parse_directory(&root)
      .unwrap()
      .manifest
      .exports
      .len(),
    2
  );

  write_source(
    &root,
    "pda.ts",
    "extern(\"pda.close\", (): void => {}); extern(\"pda\", { open: (): void => {} });",
  );

  assert!(ExternManifestParser::new().parse_directory(&root).is_err());

  write_source(
    &root,
    "pda.ts",
    "extern(\"pda\", (): void => {}); extern(\"pda.close\", (): void => {});",
  );

  assert!(ExternManifestParser::new().parse_directory(&root).is_err());
}

// Names that only share a root are written into the one table side by side, which is what dotted externs are for.
#[test]
fn keeps_dotted_externs_that_share_only_a_root() {
  let root: PathBuf = create_test_root("shared-root");

  write_source(&root, "give.ts", "extern(\"xr_effects.give_item\", (): void => {});");
  write_source(
    &root,
    "take.ts",
    "extern(\"xr_effects.take_item\", (): void => {}); extern(\"xr_effects_extra\", 1 as number);",
  );

  let parsed = ExternManifestParser::new().parse_directory(&root).unwrap();

  assert_eq!(parsed.manifest.exports.len(), 3);
}

#[test]
fn reports_malformed_typescript_instead_of_panicking() {
  let root: PathBuf = create_test_root("malformed");
  write_source(&root, "broken.ts", "export {}; extern(\"test\", (): boolean => );");

  let error: String = ExternManifestParser::new()
    .parse_directory(&root)
    .unwrap_err()
    .to_string();

  assert!(
    error.starts_with("Parsing error: Failed to parse TypeScript "),
    "{error}"
  );
  assert!(error.contains("broken.ts:1:42"), "{error}");
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
fn resolves_imports_through_the_most_specific_path_alias() {
  let root: PathBuf = create_test_root("overlapping-aliases");

  write_source(
    &root,
    "tsconfig.json",
    r#"{
      "compilerOptions": {
        "paths": {
          "@/*": ["src/*"],
          "@/shared/*": ["shared/*"]
        }
      }
    }"#,
  );
  write_source(
    &root,
    "src/shared/callbacks.ts",
    "export function run(generic: number): boolean { return true; }",
  );
  write_source(
    &root,
    "shared/callbacks.ts",
    "export function run(specific: string): void {}",
  );
  write_source(
    &root,
    "src/declarations/callbacks.ts",
    r#"
      import { run } from "@/shared/callbacks";

      extern("callbacks", { run: run });
    "#,
  );

  let parsed = ExternManifestParser::new()
    .parse_directory(&root.join("src/declarations"))
    .unwrap();
  let ExternExport::Callable(callback) = parsed.manifest.exports.get("callbacks.run").unwrap() else {
    panic!("Expected callable extern");
  };

  assert_eq!(callback.params[0].name, "specific");
  assert_eq!(callback.params[0].type_name, "string");
  assert_eq!(callback.returns, "void");
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
fn reads_the_extension_whatever_case_the_host_recorded_it_in() {
  // A declaration a Windows tool wrote back as `Example.TS` still contributes.
  assert!(ExternManifestParser::is_source_path(Path::new(
    "declarations/Example.TS"
  )));
  // Still not every name ending in the letters: the splitter takes the token after the final dot.
  assert!(!ExternManifestParser::is_source_path(Path::new("declarations/myts")));
}

#[test]
fn rejects_a_missing_source_root() {
  let root: PathBuf = GeneratedResources::path("extern-parser/missing");

  // A root that cannot be walked is the environment refusing, never a declaration to report on.
  let error: XrfError = ExternManifestParser::new().parse_directory(&root).unwrap_err();

  assert!(error.to_string().contains("is not a directory"), "{error}");
  assert!(matches!(error, XrfError::Io { .. }), "{error}");
}

/// A source the scan cannot read fails it, rather than leaving a manifest that silently lacks its externs.
#[test]
fn rejects_a_source_it_cannot_read() {
  let root: PathBuf = create_test_root("unreadable");

  fs::write(root.join("externs.ts"), b"export {}; extern(\"x\", () => {}); \xff").unwrap();

  let error: XrfError = ExternManifestParser::new().parse_directory(&root).unwrap_err();

  assert!(error.to_string().contains("Failed to read extern source"), "{error}");
  assert!(matches!(error, XrfError::Io { .. }), "{error}");
}
