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
use crate::generated_output::{normalize_doc_indentation, write_generated};
use crate::mirror_imports::MirrorImports;
use crate::type_modules::TypeModules;
use crate::type_ownership::TypeOwnership;
use crate::typescript::format::TypeScriptFormat;

/// Writes `types/`, one module per crate that declares an exported type or constant.
pub(crate) struct TypeModuleWriter;

impl TypeModuleWriter {
  /// The largest integer a JavaScript number holds exactly, `Number.MAX_SAFE_INTEGER`.
  const MAX_SAFE_INTEGER: u64 = (1 << 53) - 1;

  /// Writes one module per declaring crate, or crate family as `type_modules` groups them, each importing another as
  /// `imports` addresses it, answering with the ownership and the enums those modules establish.
  ///
  /// A crate's constants marked `#[xrf_ipc::constant]` open its module, after the imports and before the types.
  pub(crate) fn write_all(
    output: &Path,
    collected: &Types,
    type_modules: &TypeModules,
    imports: &MirrorImports,
  ) -> (TypeOwnership, Enumerations) {
    let types: Types = TypeScriptFormat::default()
      .map_types(collected)
      .expect("Failed to apply the TypeScript format to the collected types")
      .into_owned();
    let exporter: Exporter = CommandTypescript::configured().into();
    let enumerations: Enumerations = Enumerations::resolve(collected, &types);
    let (mut ownership, modules) = TypeOwnership::resolve(&types, type_modules, imports);

    for (module, named) in &modules {
      for declared in named {
        if let Some(name) = enumerations.enum_name_of(&declared.name) {
          ownership.declare(name, module);
        }
      }
    }

    let constants: BTreeMap<String, Vec<&IpcConstant>> = Self::group_constants(type_modules);
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

  /// One type as the frontend declares it, which is Specta's rendering, its doc indentation normalized, plus whatever
  /// enum the type declares.
  fn render_declaration(
    exporter: &Exporter,
    types: &Types,
    enumerations: &Enumerations,
    declared: &NamedDataType,
    module: &str,
  ) -> String {
    let union: String = primitives::export(&exporter, types, iter::once(declared), "")
      .unwrap_or_else(|error| panic!("Failed to export {module} type {}: {error}", declared.name));

    normalize_doc_indentation(&enumerations.render(declared, &union))
  }

  /// Every registered constant by the module of the crate declaring it, each module's sorted by name.
  ///
  /// Fails on a constant no workspace crate declares, and on two of one name in one module.
  fn group_constants(type_modules: &TypeModules) -> BTreeMap<String, Vec<&'static IpcConstant>> {
    let mut grouped: BTreeMap<String, Vec<&IpcConstant>> = BTreeMap::new();

    for constant in IpcConstant::list() {
      let module: String = type_modules.module_of(constant.module_path).unwrap_or_else(|| {
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
  /// A string is its own literal (`export const NAME = "value" as const;`), so the page can name one kind of value (a
  /// job kind) by the union of the constants declaring them; a number or a boolean is typed as its primitive.
  ///
  /// Fails on a name a type of the module already takes, on a value that does not serialize, on one neither a number,
  /// a string nor a boolean, and on an integer a JavaScript number does not hold exactly.
  fn render_constant(constant: &IpcConstant, module: &str, ownership: &TypeOwnership) -> String {
    assert!(
      !ownership.is_declared_by(constant.name, module),
      "Constant `{}` takes the name of a type {module} declares",
      constant.name
    );

    let value: serde_json::Value = (constant.value)().unwrap_or_else(|error| {
      panic!(
        "Constant `{}` of `{}` does not serialize: {error}",
        constant.name, constant.module_path
      )
    });
    let doc: String = Self::render_doc(constant.doc);

    let kind: &str = match &value {
      // Its own literal, by assertion rather than annotation, as the frontend's lint asks of a literal type.
      serde_json::Value::String(_) => return format!("{doc}export const {} = {value} as const;\n", constant.name),
      serde_json::Value::Number(number) => {
        assert!(
          number
            .as_i64()
            .map(i64::unsigned_abs)
            .or(number.as_u64())
            .is_none_or(|magnitude| magnitude <= Self::MAX_SAFE_INTEGER),
          "Constant `{}` of `{}` is {number}, an integer a JavaScript number does not hold exactly",
          constant.name,
          constant.module_path
        );

        "number"
      }
      serde_json::Value::Bool(_) => "boolean",
      other => panic!(
        "Constant `{}` of {module} serializes to {other}, which is not a number, a string or a boolean",
        constant.name
      ),
    };

    format!("{doc}export const {}: {kind} = {value};\n", constant.name)
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
  use xrf_ipc::IpcConstant;
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use crate::mirror_imports::MirrorImports;
  use crate::type_module_writer::TypeModuleWriter;
  use crate::type_modules::TypeModules;
  use crate::type_ownership::TypeOwnership;

  const IMPORTS: MirrorImports = MirrorImports::new("@/fixture");

  /// One constant rendered alone, into a module with no types.
  fn render(constant: &IpcConstant) -> String {
    let (ownership, _) = TypeOwnership::resolve(&Types::default(), &TypeModules::default(), &IMPORTS);

    TypeModuleWriter::render_constant(constant, "xrf-fixture", &ownership)
  }

  /// Steps a test exports to its crate's module.
  ///
  /// The second line closes no comment: */
  #[xrf_ipc::constant]
  const EXPORTED_STEPS: u32 = 7;

  /// A name the test exports beside the steps.
  #[xrf_ipc::constant]
  const EXPORTED_NAME: &str = "steps";

  /// A pattern the test exports as written:
  ///   its indent and its ` *  ` kept.
  #[xrf_ipc::constant]
  const EXPORTED_PATTERN: &str = "a *  b";

  #[test]
  fn writes_a_marked_constant_into_its_crate_s_module_with_its_doc() {
    let output: PathBuf = build_absolute_generated_test_resource_path("type-module-writer");

    fs::create_dir_all(&output).expect("directory to be created");

    TypeModuleWriter::write_all(&output, &Types::default(), &TypeModules::default(), &IMPORTS);

    let module: String = fs::read_to_string(output.join("xrf-ipc-typescript.ts")).expect("module to be written");

    assert_eq!(EXPORTED_STEPS, 7);
    assert_eq!(EXPORTED_NAME, "steps");
    assert_eq!(EXPORTED_PATTERN, "a *  b");
    assert_eq!(
      module,
      "// Auto-generated rust bindings. Do not edit it manually.\n\n\
       /** A name the test exports beside the steps. */\n\
       export const EXPORTED_NAME = \"steps\" as const;\n\n\
       /**\n \
       * A pattern the test exports as written:\n \
       *   its indent and its ` *  ` kept.\n \
       */\n\
       export const EXPORTED_PATTERN = \"a *  b\" as const;\n\n\
       /**\n \
       * Steps a test exports to its crate's module.\n \
       *\n \
       * The second line closes no comment: *\\/\n \
       */\n\
       export const EXPORTED_STEPS: number = 7;\n"
    );
  }

  #[test]
  fn writes_an_f32_in_its_shortest_form() {
    let rendered: String = render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "SHARE",
      doc: "",
      value: || xrf_ipc::to_value(&0.1_f32),
    });

    assert_eq!(rendered, "export const SHARE: number = 0.1;\n");
  }

  #[test]
  fn writes_the_largest_integer_a_javascript_number_holds() {
    let rendered: String = render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "LARGEST",
      doc: "",
      value: || xrf_ipc::to_value(&-9_007_199_254_740_991_i64),
    });

    assert_eq!(rendered, "export const LARGEST: number = -9007199254740991;\n");
  }

  #[test]
  #[should_panic(expected = "Constant `HUGE` of `xrf_fixture::limits` is 9007199254740992, an integer a JavaScript")]
  fn refuses_an_integer_a_javascript_number_does_not_hold() {
    render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "HUGE",
      doc: "",
      value: || xrf_ipc::to_value(&9_007_199_254_740_992_u64),
    });
  }

  #[test]
  #[should_panic(expected = "Constant `BROKEN` of `xrf_fixture::limits` does not serialize")]
  fn names_a_constant_that_does_not_serialize() {
    render(&IpcConstant {
      module_path: "xrf_fixture::limits",
      name: "BROKEN",
      doc: "",
      value: || serde_json::from_str("not a value"),
    });
  }
}
