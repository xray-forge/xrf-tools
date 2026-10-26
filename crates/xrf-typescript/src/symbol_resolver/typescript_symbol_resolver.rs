use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::XrfResult;

use super::typescript_project::TypeScriptProject;
use super::typescript_symbol::TypeScriptSymbol;
use crate::TypeScriptSource;
use crate::swc_common::SourceMap;
use crate::swc_ecma_ast::{ExportSpecifier, ImportSpecifier, ModuleDecl, ModuleItem, NamedExport, Program};

/// Resolves the contracts of the symbols TypeScript sources use within one project.
///
/// The resolver follows local declarations, named imports, configured path aliases, and named, local and wildcard
/// exports.
pub struct TypeScriptSymbolResolver {
  project: TypeScriptProject,
}

impl TypeScriptSymbolResolver {
  /// Create a resolver by discovering the nearest TypeScript project configuration.
  pub fn discover(source_root: &Path) -> XrfResult<Self> {
    Ok(Self {
      project: TypeScriptProject::discover(source_root)?,
    })
  }

  /// Resolve the symbol `local_name` names in a parsed source: its own declaration, or the export it imports.
  ///
  /// Returns `None` when the name resolves to no declaration with a readable contract; an error reading or parsing a
  /// module the resolution reaches is an error.
  pub fn resolve_symbol(
    &self,
    source_file: &Path,
    source_map: &SourceMap,
    program: &Program,
    local_name: &str,
  ) -> XrfResult<Option<TypeScriptSymbol>> {
    self.resolve_binding(source_file, source_map, program, local_name, &mut BTreeSet::new())
  }

  /// Resolve the type of a named property on a referenced object literal.
  pub fn resolve_member_type(
    &self,
    source_file: &Path,
    source_map: &SourceMap,
    program: &Program,
    object_name: &str,
    property_name: &str,
  ) -> XrfResult<Option<String>> {
    let symbol: Option<TypeScriptSymbol> = self.resolve_symbol(source_file, source_map, program, object_name)?;

    Ok(symbol.and_then(|symbol| symbol.property_type(property_name).map(String::from)))
  }

  /// Resolve a name bound in one module, by its own declaration or by an import.
  fn resolve_binding(
    &self,
    source_file: &Path,
    source_map: &SourceMap,
    program: &Program,
    name: &str,
    visited: &mut BTreeSet<(PathBuf, String)>,
  ) -> XrfResult<Option<TypeScriptSymbol>> {
    if let Some(symbol) = TypeScriptSymbol::find_declared(program, name, source_map) {
      return Ok(Some(symbol));
    }

    let Some((specifier, export_name)) = Self::find_import(program, name) else {
      return Ok(None);
    };

    match self.project.resolve_module_path(source_file, &specifier) {
      Some(module_path) => self.resolve_export(&module_path, &export_name, visited),
      None => Ok(None),
    }
  }

  /// Resolve one name a module exports, through its declarations, named exports and wildcard re-exports.
  ///
  /// A name a module exports explicitly is bound by that export alone, so wildcard re-exports are searched only after
  /// every explicit one. `visited` holds each module and name already asked, which ends a re-export cycle unresolved.
  fn resolve_export(
    &self,
    module_path: &Path,
    export_name: &str,
    visited: &mut BTreeSet<(PathBuf, String)>,
  ) -> XrfResult<Option<TypeScriptSymbol>> {
    // Canonical, so that two spellings of one module, `./a` and `../dir/a`, meet in a cycle.
    let module_key: PathBuf = fs::canonicalize(module_path).unwrap_or_else(|_| module_path.to_path_buf());

    if !visited.insert((module_key, export_name.to_owned())) {
      return Ok(None);
    }

    let source: TypeScriptSource = TypeScriptSource::parse_file(module_path)?;
    let Program::Module(module) = &source.program else {
      return Ok(None);
    };

    for item in &module.body {
      match item {
        ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => {
          if let Some(symbol) = TypeScriptSymbol::from_declaration(&export.decl, export_name, &source.source_map) {
            return Ok(Some(symbol));
          }
        }
        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named)) => {
          let Some(original_name) = Self::find_reexported_name(named, export_name) else {
            continue;
          };

          return match &named.src {
            Some(specifier) => {
              match self
                .project
                .resolve_module_path(module_path, &specifier.value.to_string_lossy())
              {
                Some(path) => self.resolve_export(&path, &original_name, visited),
                None => Ok(None),
              }
            }
            None => self.resolve_binding(
              module_path,
              &source.source_map,
              &source.program,
              &original_name,
              visited,
            ),
          };
        }
        _ => {}
      }
    }

    for item in &module.body {
      let ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export_all)) = item else {
        continue;
      };
      let Some(path) = self
        .project
        .resolve_module_path(module_path, &export_all.src.value.to_string_lossy())
      else {
        continue;
      };

      if let Some(symbol) = self.resolve_export(&path, export_name, visited)? {
        return Ok(Some(symbol));
      }
    }

    Ok(None)
  }

  /// The module specifier and exported name a value import binds to `local_name`.
  fn find_import(program: &Program, local_name: &str) -> Option<(String, String)> {
    let Program::Module(module) = program else {
      return None;
    };

    for item in &module.body {
      let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item else {
        continue;
      };

      // `import type { X }` binds no value, whatever its specifiers say one by one.
      if import.type_only {
        continue;
      }

      for specifier in &import.specifiers {
        let ImportSpecifier::Named(named) = specifier else {
          continue;
        };

        if named.is_type_only || named.local.sym != *local_name {
          continue;
        }

        let export_name: String = named
          .imported
          .as_ref()
          .map_or_else(|| local_name.into(), |imported| imported.atom().to_string());

        return Some((import.src.value.to_string_lossy().to_string(), export_name));
      }
    }

    None
  }

  /// The name a named export binds where it exports a value as `export_name`.
  fn find_reexported_name(named: &NamedExport, export_name: &str) -> Option<String> {
    // `export type { X } from` exports no value either.
    if named.type_only {
      return None;
    }

    named.specifiers.iter().find_map(|specifier| {
      let ExportSpecifier::Named(specifier) = specifier else {
        return None;
      };

      if specifier.is_type_only {
        return None;
      }

      let exposed_name = specifier.exported.as_ref().unwrap_or(&specifier.orig).atom();

      (*exposed_name == *export_name).then(|| specifier.orig.atom().to_string())
    })
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_test_utils::GeneratedResources;

  use super::TypeScriptSymbolResolver;
  use crate::swc_ecma_ast::{ModuleDecl, ModuleItem, NamedExport, Program};
  use crate::{TypeScriptFunctionParameter, TypeScriptObjectProperty, TypeScriptSource, TypeScriptSymbol};

  fn create_test_root(name: &str) -> PathBuf {
    GeneratedResources::create_dir(&format!("typescript-symbol-resolver/{name}")).unwrap()
  }

  fn write_source(root: &Path, name: &str, source: &str) {
    let path: PathBuf = root.join(name);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
  }

  /// The symbol `name` names in the root's `externs.ts`.
  fn resolve(root: &Path, name: &str) -> Option<TypeScriptSymbol> {
    let path: PathBuf = root.join("externs.ts");
    let source: TypeScriptSource = TypeScriptSource::parse_file(&path).unwrap();

    TypeScriptSymbolResolver::discover(root)
      .unwrap()
      .resolve_symbol(&path, &source.source_map, &source.program, name)
      .unwrap()
  }

  fn parse(name: &str, source: &str) -> Program {
    let root: PathBuf = create_test_root(name);

    write_source(&root, "externs.ts", source);

    TypeScriptSource::parse_file(&root.join("externs.ts")).unwrap().program
  }

  fn first_named_export(program: &Program) -> NamedExport {
    let Program::Module(module) = program else {
      panic!("Expected a module");
    };

    module
      .body
      .iter()
      .find_map(|item| match item {
        ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named)) => Some(named.clone()),
        _ => None,
      })
      .expect("Expected a named export")
  }

  #[test]
  fn follows_a_value_import_and_not_a_type_only_one() {
    let program: Program = parse(
      "imports",
      "import { run } from \"./run\";\nimport type { Config } from \"./config\";\nimport { type Shape } from \"./shape\";",
    );

    assert_eq!(
      TypeScriptSymbolResolver::find_import(&program, "run"),
      Some((String::from("./run"), String::from("run")))
    );
    assert_eq!(TypeScriptSymbolResolver::find_import(&program, "Config"), None);
    assert_eq!(TypeScriptSymbolResolver::find_import(&program, "Shape"), None);
  }

  #[test]
  fn follows_a_value_reexport_and_not_a_type_only_one() {
    let value: Program = parse("reexport-value", "export { run as start } from \"./run\";");
    let type_only: Program = parse("reexport-type", "export type { Config } from \"./config\";");

    assert_eq!(
      TypeScriptSymbolResolver::find_reexported_name(&first_named_export(&value), "start"),
      Some(String::from("run"))
    );
    assert_eq!(
      TypeScriptSymbolResolver::find_reexported_name(&first_named_export(&type_only), "Config"),
      None
    );
  }

  #[test]
  fn reads_rest_and_destructured_parameters_with_their_annotations() {
    let root: PathBuf = create_test_root("parameters");

    write_source(
      &root,
      "handlers.ts",
      "export function run([section, count]: [string, number], label = \"x\", ...rest: number[]): void {}",
    );
    write_source(&root, "externs.ts", "import { run } from \"./handlers\";");

    let Some(TypeScriptSymbol::Callable(signature)) = resolve(&root, "run") else {
      panic!("Expected a callable");
    };

    assert_eq!(
      signature.params,
      [
        TypeScriptFunctionParameter {
          name: String::from("[section, count]"),
          optional: false,
          type_name: String::from("[string, number]"),
        },
        TypeScriptFunctionParameter {
          name: String::from("label"),
          optional: true,
          type_name: String::from("unknown"),
        },
        TypeScriptFunctionParameter {
          name: String::from("rest"),
          optional: false,
          type_name: String::from("Array<number>"),
        },
      ]
    );
  }

  #[test]
  fn follows_a_local_export_of_a_declaration_and_of_an_import() {
    let root: PathBuf = create_test_root("local-export");

    write_source(&root, "run.ts", "export const run = (value: string): boolean => true;");
    write_source(
      &root,
      "index.ts",
      "import { run } from \"./run\";\nconst limit: number = 4;\nexport { limit as LIMIT, run as start };",
    );
    write_source(&root, "externs.ts", "import { LIMIT, start } from \"./index\";");

    assert_eq!(
      resolve(&root, "LIMIT"),
      Some(TypeScriptSymbol::Value(String::from("number")))
    );
    assert_eq!(
      resolve(&root, "start").map(|symbol| symbol.type_name()),
      Some(String::from("(value: string) => boolean"))
    );
  }

  #[test]
  fn prefers_an_explicit_export_over_an_earlier_wildcard() {
    let root: PathBuf = create_test_root("explicit-over-wildcard");

    write_source(&root, "wildcard.ts", "export const value: string = \"\";");
    write_source(&root, "explicit.ts", "export const value: number = 1;");
    write_source(
      &root,
      "index.ts",
      "export * from \"./wildcard\";\nexport { value } from \"./explicit\";",
    );
    write_source(&root, "externs.ts", "import { value } from \"./index\";");

    assert_eq!(
      resolve(&root, "value"),
      Some(TypeScriptSymbol::Value(String::from("number")))
    );
  }

  #[test]
  fn revisits_a_module_for_another_name_and_ends_a_cycle() {
    let root: PathBuf = create_test_root("cycles");

    write_source(
      &root,
      "a/first.ts",
      "export { second as entry } from \"../b/second\";\nexport const target: number = 1;\nexport * from \"../b/second\";",
    );
    write_source(
      &root,
      "b/second.ts",
      "export { target as second } from \"../a/first\";\nexport * from \"../a/first\";",
    );
    write_source(&root, "externs.ts", "import { entry, missing } from \"./a/first\";");

    // `first` is asked for `entry`, then again for `target`, which it declares.
    assert_eq!(
      resolve(&root, "entry"),
      Some(TypeScriptSymbol::Value(String::from("number")))
    );
    // Each module re-exports the other wholesale, through a parent-relative path that never repeats as written.
    assert_eq!(resolve(&root, "missing"), None);
  }

  #[test]
  fn types_an_unannotated_variable_by_its_assertion_or_object() {
    let root: PathBuf = create_test_root("unannotated");

    // Without imports or exports the source is a script, whose declarations resolve like a module's.
    write_source(
      &root,
      "externs.ts",
      "const table = raw as Record<string, number>;\nconst config = { enabled: (): boolean => true, \"name\": \"x\", count: 1 } as const;\nconst bare = 4;",
    );

    assert_eq!(
      resolve(&root, "table"),
      Some(TypeScriptSymbol::Value(String::from("Record<string, number>")))
    );
    assert_eq!(
      resolve(&root, "config"),
      Some(TypeScriptSymbol::Object(vec![
        TypeScriptObjectProperty {
          name: String::from("enabled"),
          type_name: String::from("() => boolean"),
        },
        TypeScriptObjectProperty {
          name: String::from("name"),
          type_name: String::from("string"),
        },
        TypeScriptObjectProperty {
          name: String::from("count"),
          type_name: String::from("number"),
        },
      ]))
    );
    assert_eq!(resolve(&root, "bare"), None);
  }
}
