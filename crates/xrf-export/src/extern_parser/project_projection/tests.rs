use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use super::{ExportContractDescriptor, ExportsProjectParser};
use crate::render::{ExternFormat, render_extern_manifest};

fn create_test_root(name: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("project-projection/{name}"));
  fs::create_dir_all(&root).unwrap();
  root
}

fn write_source(root: &Path, name: &str, source: &str) {
  let path: PathBuf = root.join(name);

  fs::create_dir_all(path.parent().unwrap()).unwrap();
  fs::write(path, source).unwrap();
}

#[test]
fn parses_all_project_exports_from_the_project_root() {
  let root: PathBuf = create_test_root("project");

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
  write_source(
    &root,
    "src/engine/core/check.ts",
    "export function check(value: string): boolean { return value.length > 0; }",
  );
  write_source(
    &root,
    "src/engine/declarations/conditions/check.ts",
    r#"
      import { check } from "@/engine/core/check";
      extern("xr_conditions.check", check);
    "#,
  );
  write_source(
    &root,
    "src/engine/declarations/effects/run.ts",
    "export {}; extern(\"xr_effects.run\", (): void => {});",
  );
  write_source(
    &root,
    "src/engine/declarations/dialogs/hello.ts",
    "export {}; extern(\"dialogs_zaton.hello\", (): void => {});",
  );
  write_source(
    &root,
    "src/engine/scripts/start.ts",
    r#"
      export {};
      extern("settings", rawValue as Record<string, boolean>);
      extern("start", (): void => {});
    "#,
  );
  write_source(
    &root,
    "src/engine/core/decorated.ts",
    "@UnsupportedDecorator() class Decorated {}",
  );
  write_source(&root, "node_modules/invalid.ts", "this is not TypeScript {{{");
  write_source(&root, "target/invalid.ts", "this is not TypeScript {{{");

  let project = ExportsProjectParser::new().parse_project_from_path(&root).unwrap();

  assert_eq!(project.root, root);
  assert_eq!(project.declarations.len(), 5);
  assert_eq!(project.declarations[0].name, "dialogs_zaton.hello");
  assert_eq!(project.declarations[1].name, "settings");
  assert!(matches!(
    &project.declarations[1].contract,
    ExportContractDescriptor::Value { typing } if typing == "Record<string, boolean>"
  ));
  assert_eq!(project.declarations[2].name, "start");
  assert_eq!(project.declarations[3].name, "xr_conditions.check");
  assert_eq!(project.declarations[4].name, "xr_effects.run");
}

#[test]
fn projects_complete_callable_and_value_contracts() {
  let root: PathBuf = create_test_root("contracts");

  write_source(
    &root,
    "declarations.ts",
    r#"
      export {};
      /**
       * Runs one task.
       * @param count - Optional repetition count.
       * @returns Whether the task ran.
       */
      extern("tasks.run", (count?: number): boolean => true);
      /** Current task settings. */
      extern("settings", rawValue as Record<string, boolean>);
    "#,
  );

  let project = ExportsProjectParser::new().parse_project_from_path(&root).unwrap();
  let json: Value = json!(project);

  assert_eq!(json["root"], root.to_string_lossy().as_ref());
  assert_eq!(json["declarations"][0]["kind"], "value");
  assert_eq!(json["declarations"][0]["typing"], "Record<string, boolean>");
  assert_eq!(json["declarations"][0]["description"], "Current task settings.");
  assert_eq!(json["declarations"][1]["kind"], "callable");
  assert_eq!(json["declarations"][1]["description"], "Runs one task.");
  assert_eq!(json["declarations"][1]["parameters"][0]["name"], "count");
  assert_eq!(json["declarations"][1]["parameters"][0]["typing"], "number");
  assert_eq!(json["declarations"][1]["parameters"][0]["isOptional"], true);
  assert_eq!(
    json["declarations"][1]["parameters"][0]["description"],
    "Optional repetition count."
  );
  assert_eq!(json["declarations"][1]["returns"]["typing"], "boolean");
  assert_eq!(
    json["declarations"][1]["returns"]["description"],
    "Whether the task ran."
  );
  assert_eq!(json["declarations"][1]["source"]["path"], "declarations.ts");
  assert!(json["declarations"][1]["source"]["line"].as_u64().unwrap() > 0);
  assert!(json["declarations"][1]["source"]["column"].as_u64().unwrap() > 0);
}

#[test]
fn carries_the_manifest_its_declarations_were_projected_from() {
  let root: PathBuf = create_test_root("manifest");

  write_source(
    &root,
    "declarations.ts",
    "export {};\nextern(\"xr_effects.run\", (): void => {});\nextern(\"settings\", rawValue as number);\n",
  );

  let project = ExportsProjectParser::new().parse_project_from_path(&root).unwrap();

  // What a surface publishes has to describe what it is showing, name for name: both are ordered by name, so the
  // artifact and the tree beside it can be read against each other.
  assert_eq!(
    project.manifest.exports.keys().cloned().collect::<Vec<String>>(),
    project
      .declarations
      .iter()
      .map(|declaration| declaration.name.clone())
      .collect::<Vec<String>>()
  );
  assert!(
    render_extern_manifest(&project.manifest, ExternFormat::Json, None)
      .unwrap()
      .contains("\"xr_effects.run\"")
  );
}

#[test]
fn keeps_an_empty_project_open() {
  let root: PathBuf = create_test_root("empty");

  let project = ExportsProjectParser::new().parse_project_from_path(&root).unwrap();

  assert_eq!(project.root, root);
  assert!(project.declarations.is_empty());
}

#[test]
fn records_the_last_line_of_a_declaration_and_reads_it_back() {
  let root: PathBuf = create_test_root("source");

  // The whole point is the multi-line case: a body is what the reader came for, and recording only
  // the opening line would show them one line of it.
  write_source(
    &root,
    "declarations.ts",
    "export {};\nextern(\"xr_effects.run\", (): void => {\n  const first: number = 1;\n\n  log(first);\n});\nconst tail: number = 2;\n",
  );

  let project = ExportsProjectParser::new().parse_project_from_path(&root).unwrap();
  let declaration = project
    .declarations
    .iter()
    .find(|declaration| declaration.name == "xr_effects.run")
    .expect("declaration is parsed");

  assert_eq!(declaration.source.line, 2);
  assert_eq!(declaration.source.end_line, 6);

  let source = project.read_declaration_source("xr_effects.run").unwrap();

  assert_eq!(
    source.content,
    "extern(\"xr_effects.run\", (): void => {\n  const first: number = 1;\n\n  log(first);\n});"
  );
  assert_eq!(source.path, "declarations.ts");
}

#[test]
fn records_the_span_of_one_property_inside_an_object_extern() {
  let root: PathBuf = create_test_root("object-source");

  // Object externs record the property span, not the whole statement, so each name reads back only
  // its own body rather than every sibling declared alongside it.
  write_source(
    &root,
    "declarations.ts",
    "export {};\nextern(\"xr_effects\", {\n  first: (): void => {\n    log(1);\n  },\n  second: (): void => {},\n});\n",
  );

  let project = ExportsProjectParser::new().parse_project_from_path(&root).unwrap();
  let source = project.read_declaration_source("xr_effects.first").unwrap();

  assert_eq!(source.content, "  first: (): void => {\n    log(1);\n  },");
}
