//! Writing `core/ipc/types/`, one module per crate that declares an exported type or constant.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::Path;

use specta::datatype::NamedDataType;
use specta::{Format, Types};
use specta_typescript::{Exporter, primitives};
use xrf_ipc::IpcConstant;

use crate::command_typescript::CommandTypescript;
use crate::constants::GENERATED_HEADER;
use crate::enumerations::Enumerations;
use crate::generated_output::write_generated;
use crate::type_ownership::TypeOwnership;
use crate::typescript::format::TypeScriptFormat;

/// Writes `types/`, one module per crate that declares an exported type or constant.
pub(crate) struct TypeModuleWriter;

impl TypeModuleWriter {
  /// Writes one module per declaring crate, answering with the ownership and the enums those modules establish.
  ///
  /// A crate's constants marked `#[xrf_ipc::constant]` open its module, after the imports and before the types.
  pub(crate) fn write_all(output: &Path, collected: &Types) -> (TypeOwnership, Enumerations) {
    let types: Types = TypeScriptFormat::default()
      .map_types(collected)
      .expect("Failed to apply the TypeScript format to the collected types")
      .into_owned();
    let exporter: Exporter = CommandTypescript::configured().into();
    let enumerations: Enumerations = Enumerations::resolve(collected, &types);
    let (mut ownership, modules) = TypeOwnership::resolve(&types);

    for (module, named) in &modules {
      for declared in named {
        if let Some(name) = enumerations.enum_name_of(&declared.name) {
          ownership.declare(name, module);
        }
      }
    }

    let constants: BTreeMap<String, Vec<&IpcConstant>> = Self::group_constants();
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut rendered: BTreeMap<String, String> = BTreeMap::new();

    for module in modules.keys().chain(constants.keys()).collect::<BTreeSet<&String>>() {
      let declarations: String = modules
        .get(module)
        .into_iter()
        .flatten()
        .map(|declared| Self::render_declaration(&exporter, &types, &enumerations, declared, module))
        .collect::<Vec<String>>()
        .join("\n");

      ownership.assert_no_foreign_references(&declarations, module);

      let referenced: BTreeSet<&str> = ownership.references(&declarations, module);

      graph.insert(module.clone(), ownership.modules_of(&referenced));

      // `primitives::export` renders declarations alone, so the file header the high-level exporter would have
      // written has to be prepended here.
      let imports: String = ownership.imports(&referenced);
      let separator: &str = if imports.is_empty() { "" } else { "\n" };
      let exported: String = constants
        .get(module)
        .into_iter()
        .flatten()
        .map(|constant| Self::render_constant(constant, module, &ownership))
        .collect::<Vec<String>>()
        .join("\n");
      let constant_separator: &str = if exported.is_empty() || declarations.is_empty() {
        ""
      } else {
        "\n"
      };

      rendered.insert(
        module.clone(),
        format!("{GENERATED_HEADER}\n{imports}{separator}{exported}{constant_separator}{declarations}"),
      );
    }

    TypeOwnership::assert_no_import_cycles(&graph);

    for (module, contents) in rendered {
      write_generated(&output.join(format!("{module}.ts")), &contents);
    }

    (ownership, enumerations)
  }

  /// One type as the frontend declares it, which is Specta's rendering plus whatever enum the type declares.
  fn render_declaration(
    exporter: &Exporter,
    types: &Types,
    enumerations: &Enumerations,
    declared: &NamedDataType,
    module: &str,
  ) -> String {
    let union: String = primitives::export(&exporter, types, iter::once(declared), "")
      .unwrap_or_else(|error| panic!("Failed to export {module} type {}: {error}", declared.name));

    enumerations.render(declared, &union)
  }

  /// Every registered constant by the module of the crate declaring it, each module's sorted by name.
  ///
  /// Fails on a constant no workspace crate declares, and on two of one name in one module.
  fn group_constants() -> BTreeMap<String, Vec<&'static IpcConstant>> {
    let mut grouped: BTreeMap<String, Vec<&IpcConstant>> = BTreeMap::new();

    for constant in IpcConstant::list() {
      let module: String = TypeOwnership::owning_module_of(constant.module_path).unwrap_or_else(|| {
        panic!(
          "Constant `{}` is declared in `{}`, outside the workspace, and has no module to live in",
          constant.name, constant.module_path
        )
      });

      grouped.entry(module).or_default().push(constant);
    }

    for (module, exported) in &mut grouped {
      exported.sort_by_key(|constant| constant.name);

      for pair in exported.windows(2) {
        assert_ne!(
          pair[0].name, pair[1].name,
          "Constant `{}` is exported twice into {module}, from `{}` and `{}`",
          pair[0].name, pair[0].module_path, pair[1].module_path
        );
      }
    }

    grouped
  }

  /// One constant as the frontend declares it: its doc comment, then `export const NAME: type = value;`.
  ///
  /// Fails on a name a type of the module already takes, and on a value neither a number, a string nor a boolean.
  fn render_constant(constant: &IpcConstant, module: &str, ownership: &TypeOwnership) -> String {
    assert!(
      !ownership.is_declared_by(constant.name, module),
      "Constant `{}` takes the name of a type {module} declares",
      constant.name
    );

    let value: serde_json::Value = (constant.value)();
    let kind: &str = match &value {
      serde_json::Value::Number(_) => "number",
      serde_json::Value::String(_) => "string",
      serde_json::Value::Bool(_) => "boolean",
      other => panic!(
        "Constant `{}` of {module} serializes to {other}, which is not a number, a string or a boolean",
        constant.name
      ),
    };

    format!(
      "{}export const {}: {kind} = {value};\n",
      Self::render_doc(constant.doc),
      constant.name
    )
  }

  /// A doc comment as JSDoc, on one line where it is one line; nothing where there is none.
  fn render_doc(doc: &str) -> String {
    let escaped: String = doc.replace("*/", "*\\/");
    let lines: Vec<&str> = escaped.lines().collect();

    match lines.as_slice() {
      [] => String::new(),
      [line] => format!("/** {line} */\n"),
      lines => {
        let body: String = lines
          .iter()
          .map(|line| {
            if line.is_empty() {
              String::from(" *\n")
            } else {
              format!(" * {line}\n")
            }
          })
          .collect();

        format!("/**\n{body} */\n")
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use specta::Types;
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use crate::type_module_writer::TypeModuleWriter;

  /// Steps a test exports to its crate's module.
  ///
  /// The second line closes no comment: */
  #[xrf_ipc::constant]
  const EXPORTED_STEPS: u32 = 7;

  /// A name the test exports beside the steps.
  #[xrf_ipc::constant]
  const EXPORTED_NAME: &str = "steps";

  #[test]
  fn writes_a_marked_constant_into_its_crate_s_module_with_its_doc() {
    let output: PathBuf = build_absolute_generated_test_resource_path("type-module-writer");

    fs::create_dir_all(&output).expect("directory to be created");

    TypeModuleWriter::write_all(&output, &Types::default());

    let module: String = fs::read_to_string(output.join("xrf-ipc-typescript.ts")).expect("module to be written");

    assert_eq!(EXPORTED_STEPS, 7);
    assert_eq!(EXPORTED_NAME, "steps");
    assert_eq!(
      module,
      "// Auto-generated rust bindings. Do not edit it manually.\n\n\
       /** A name the test exports beside the steps. */\n\
       export const EXPORTED_NAME: string = \"steps\";\n\n\
       /**\n \
       * Steps a test exports to its crate's module.\n \
       *\n \
       * The second line closes no comment: *\\/\n \
       */\n\
       export const EXPORTED_STEPS: number = 7;\n"
    );
  }
}
